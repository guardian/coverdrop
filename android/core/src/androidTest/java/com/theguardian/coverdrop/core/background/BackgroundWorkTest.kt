package com.theguardian.coverdrop.core.background

import android.app.Application
import android.content.Context
import android.util.Log
import androidx.test.platform.app.InstrumentationRegistry
import androidx.work.Configuration
import androidx.work.ListenableWorker.Result
import androidx.work.WorkInfo
import androidx.work.WorkManager
import androidx.work.testing.SynchronousExecutor
import androidx.work.testing.TestDriver
import androidx.work.testing.TestListenableWorkerBuilder
import androidx.work.testing.WorkManagerTestInitHelper
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.CoverDropLibInternalFixture
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.integrationtests.createEncryptedStorageForTest
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace
import com.theguardian.coverdrop.core.persistence.PublicStorage
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestApiCallProvider
import com.theguardian.coverdrop.testutils.TestClock
import com.theguardian.coverdrop.testutils.TestScenario
import com.theguardian.coverdrop.testutils.createCoverDropConfigurationForTest
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Before
import org.junit.Test
import java.time.Duration

class BackgroundWorkTest {
    val context: Context = InstrumentationRegistry.getInstrumentation().targetContext
    private val scenario = TestScenario.Minimal

    private val testVectors = IntegrationTestVectors(context, scenario)
    private val fileManager = CoverDropFileManager(context, CoverDropNamespace.TEST)
    private val config = createCoverDropConfigurationForTest(context, scenario)
    private val clock = config.clock as TestClock
    private val publicStorage = PublicStorage(context, fileManager)
    private val testApiCallProvider = config.createApiCallProvider() as TestApiCallProvider
    private val encryptedStorage = createEncryptedStorageForTest(context, fileManager)

    // initialize the main library components with the mocks
    private val lib = CoverDropLibInternalFixture(
        mApiCallProvider = testApiCallProvider,
        mContext = context.applicationContext as Application,
        mConfig = config,
        mClock = clock,
        mEncryptedStorage = encryptedStorage,
        mLibSodium = createLibSodium(),
        mPublicStorage = publicStorage,
    )

    private lateinit var workManager: WorkManager
    private lateinit var workManagerTestDriver: TestDriver
    private lateinit var backgroundWorkManager: BackgroundWorkManager

    @Before
    fun setup(): Unit = runBlocking {
        CoverDropBackgroundWorker.overrideInternalLibForTesting(lib)
        clock.setNow(testVectors.getNow())
        lib.initialize()
        testApiCallProvider.clearLoggedPostRequests()

        val workManagerConfiguration = Configuration.Builder()
            .setMinimumLoggingLevel(Log.VERBOSE)
            .setExecutor(SynchronousExecutor())
            .build()
        WorkManagerTestInitHelper.initializeTestWorkManager(context, workManagerConfiguration)
        workManagerTestDriver = WorkManagerTestInitHelper.getTestDriver(context)!!
        workManager = WorkManager.getInstance(context)
        backgroundWorkManager = BackgroundWorkManager(lib, context, workManager)

        assertThat(getAllCoverDropWorkInfos()).isEmpty()
    }

    @After
    fun tearDown(): Unit = runBlocking {
        CoverDropBackgroundWorker.overrideInternalLibForTesting(null)
        publicStorage.deleteAll()
    }

    @Test
    fun testScheduling_whenOnAppStart_thenEnqueuedAndFlagSet(): Unit = runBlocking {
        backgroundWorkManager.onAppStart()
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)
        assertThat(publicStorage.readBackgroundWorkPending()).isTrue()
    }

    @Test
    fun testScheduling_whenOnAppFinished_thenEnqueuedAndFlagSet(): Unit = runBlocking {
        backgroundWorkManager.onAppFinished()
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)
        assertThat(publicStorage.readBackgroundWorkPending()).isTrue()
    }

    @Test
    fun testScheduling_simple(): Unit = runBlocking {
        // after closing the app there is exactly one enqueued work task
        backgroundWorkManager.onAppFinished()
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)
        assertMessagesSent(numMessages = 0)

        val lastRequestId = getAllCoverDropWorkInfos().single().id
        workManagerTestDriver.setAllConstraintsMet(lastRequestId)
        workManagerTestDriver.setInitialDelayMet(lastRequestId)
        waitForAllCoverDropWorkersToFinishRunning()

        // after running, there is actually one finished work task
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.SUCCEEDED)

        // and we posted two messages
        assertMessagesSent(numMessages = 2)
    }

    @Test
    fun testScheduling_whenDidNotExecuteInBackground_thenExecutedOnAppStart(): Unit = runBlocking {
        // after closing the app there is exactly one enqueued work task
        backgroundWorkManager.onAppFinished()
        val originalRequestId = getAllCoverDropWorkInfos().single().id
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)

        // we have not posted any messages yet
        assertThat(testApiCallProvider.getLoggedPostRequestEndpoints()).isEmpty()

        // after starting the app we sent the messages
        backgroundWorkManager.onAppStart()
        assertMessagesSent(numMessages = 2)

        // the work task is being cancelled and replaced with a new one
        val newRequestId = getAllCoverDropWorkInfos().single().id
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)

        // the new request is different from the original one
        assertThat(newRequestId).isNotEqualTo(originalRequestId)
    }

    @Test
    fun testScheduling_whenExecutedInBackground_thenNotExecutedOnAppStart(): Unit = runBlocking {
        // after closing the app there is exactly one enqueued work task
        backgroundWorkManager.onAppFinished()
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)

        val lastRequestId = getAllCoverDropWorkInfos().single().id
        workManagerTestDriver.setAllConstraintsMet(lastRequestId)
        workManagerTestDriver.setInitialDelayMet(lastRequestId)
        waitForAllCoverDropWorkersToFinishRunning()

        // after running, there is actually one finished work task
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.SUCCEEDED)

        // and we posted two messages
        assertMessagesSent(numMessages = 2)

        // after starting the app we should not send the messages again
        backgroundWorkManager.onAppStart()
        assertMessagesSent(numMessages = 2)

        // but there is still a new one enqueued
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)
    }

    @Test
    fun testScheduling_whenOnlyExecutingOnAppStart_thenStillScheduled(): Unit = runBlocking {
        // after starting the app there is exactly one enqueued work task
        backgroundWorkManager.onAppStart()
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)

        // tasks executes once conditions are met
        val lastRequestId = getAllCoverDropWorkInfos().single().id
        workManagerTestDriver.setAllConstraintsMet(lastRequestId)
        workManagerTestDriver.setInitialDelayMet(lastRequestId)
        waitForAllCoverDropWorkersToFinishRunning()

        // after running, there is actually one finished work task
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.SUCCEEDED)
    }

    @Test
    fun testScheduling_whenOnlyExecutingOnAppFinished_thenStillScheduled(): Unit = runBlocking {
        // after closing the app there is exactly one enqueued work task
        backgroundWorkManager.onAppFinished()
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)

        // tasks executes once conditions are met
        val lastRequestId = getAllCoverDropWorkInfos().single().id
        workManagerTestDriver.setAllConstraintsMet(lastRequestId)
        workManagerTestDriver.setInitialDelayMet(lastRequestId)
        waitForAllCoverDropWorkersToFinishRunning()

        // after running, there is actually one finished work task
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.SUCCEEDED)
    }

    @Test
    fun testScheduling_complexScenario(): Unit = runBlocking {
        // we start the app and observe a task is scheduled in the future
        backgroundWorkManager.onAppStart()
        val id1 = getAllCoverDropWorkInfos().single().id
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)

        // we close the app and observe a task is scheduled in the future
        backgroundWorkManager.onAppFinished()
        val id2 = getAllCoverDropWorkInfos().single().id
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)
        assertThat(id2).isNotEqualTo(id1)

        // we wait in the background and observe the task is executed
        workManagerTestDriver.setAllConstraintsMet(id2)
        workManagerTestDriver.setInitialDelayMet(id2)
        waitForAllCoverDropWorkersToFinishRunning()
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.SUCCEEDED)
        assertMessagesSent(numMessages = 2)

        // we start the app and observe a task is scheduled in the future
        backgroundWorkManager.onAppStart()
        val id3 = getAllCoverDropWorkInfos().single().id
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)

        // we don't call the app close method this time; the task remains scheduled
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)
        assertThat(getAllCoverDropWorkInfos().single().runAttemptCount).isEqualTo(0)

        // we wait in the background and observe the task is scheduled, but not executed because
        // it was rate limited
        workManagerTestDriver.setAllConstraintsMet(id3)
        workManagerTestDriver.setInitialDelayMet(id3)
        waitForAllCoverDropWorkersToFinishRunning()
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.ENQUEUED)
        assertThat(getAllCoverDropWorkInfos().single().runAttemptCount).isEqualTo(1)

        // move the clock forward to get past the rate limit
        clock.advance(Duration.ofHours(2))
        workManagerTestDriver.setAllConstraintsMet(id3)
        workManagerTestDriver.setInitialDelayMet(id3)
        waitForAllCoverDropWorkersToFinishRunning()
        assertThat(getAllCoverDropWorkInfos().single().state).isEqualTo(WorkInfo.State.SUCCEEDED)
        assertThat(getAllCoverDropWorkInfos().single().runAttemptCount).isEqualTo(2)
        assertMessagesSent(numMessages = 4)
    }

    @Test
    fun testRunning_simple(): Unit = runBlocking {
        val worker = TestListenableWorkerBuilder<CoverDropBackgroundWorker>(context).build();
        val run = worker.doWork()
        assertThat(run).isEqualTo(Result.success())
    }

    @Test
    fun testRunning_whenExecutedFrequently_thenRateLimitNotExceeded(): Unit = runBlocking {
        val worker = TestListenableWorkerBuilder<CoverDropBackgroundWorker>(context).build();

        val run1 = worker.doWork()
        assertThat(run1).isEqualTo(Result.success())

        clock.advance(Duration.ofSeconds(5))
        val run2 = worker.doWork()
        assertThat(run2).isEqualTo(Result.retry())

        clock.advance(Duration.ofSeconds(5))
        val run3 = worker.doWork()
        assertThat(run3).isEqualTo(Result.retry())

        clock.advance(Duration.ofHours(2))
        val run4 = worker.doWork()
        assertThat(run4).isEqualTo(Result.success())
    }

    @Test
    fun testRunning_whenClockJumpsBackwards_thenStillExecuted(): Unit = runBlocking {
        val worker = TestListenableWorkerBuilder<CoverDropBackgroundWorker>(context).build();

        val run1 = worker.doWork()
        assertThat(run1).isEqualTo(Result.success())

        clock.advance(Duration.ofHours(-1))

        val run2 = worker.doWork()
        assertThat(run2).isEqualTo(Result.success())
    }

    /**
     * Returns all [WorkInfo] instances for the [COVERDROP_BACKGROUND_WORKER_NAME] tag.
     */
    private fun getAllCoverDropWorkInfos(): List<WorkInfo> {
        val workInfos = workManager.getWorkInfosByTag(COVERDROP_BACKGROUND_WORKER_NAME).get()!!
        workInfos.forEach { Log.d("BackgroundWorkTest", "WorkInfo: $it") }
        return workInfos
    }

    /**
     * Waits until all [CoverDropBackgroundWorker] instances have finished running.
     */
    private fun waitForAllCoverDropWorkersToFinishRunning() {
        while (getAllCoverDropWorkInfos().any { it.state == WorkInfo.State.RUNNING }) {
            Thread.sleep(100)
        }
    }

    /**
     * Asserts that the given number of messages have been sent to the API. This is the cumulative
     * total number of messages sent during the test case.
     */
    private fun assertMessagesSent(numMessages: Int) {
        val expected = List(numMessages) { "/user/messages" }
        assertThat(testApiCallProvider.getLoggedPostRequestEndpoints()).isEqualTo(expected)
    }
}
