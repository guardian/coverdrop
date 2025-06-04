package com.theguardian.coverdrop.audit

import android.app.Application
import android.os.Environment
import android.util.Log
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry.getInstrumentation
import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.CoverDropConfiguration
import com.theguardian.coverdrop.core.CoverDropLib
import com.theguardian.coverdrop.core.CoverDropThrowingExceptionHandler
import com.theguardian.coverdrop.testutils.IntegrationTestVectors
import com.theguardian.coverdrop.testutils.TestApiCallProvider
import com.theguardian.coverdrop.testutils.TestClock
import com.theguardian.coverdrop.testutils.TestScenario
import dagger.hilt.android.testing.HiltAndroidRule
import dagger.hilt.android.testing.HiltAndroidTest
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.nio.file.attribute.PosixFilePermission
import kotlin.io.path.getOwner
import kotlin.io.path.getPosixFilePermissions
import kotlin.time.Duration.Companion.seconds


/**
 * This "self audit" ensures that we do not add any unaccounted stored files.
 */
@HiltAndroidTest
@RunWith(AndroidJUnit4::class)
class StorageAuditTest {

    @get:Rule
    var hiltRule = HiltAndroidRule(this)

    private val targetContext = getInstrumentation().targetContext

    @Before
    fun setup() {
        hiltRule.inject()
    }

    /**
     * All path prefixes to ignore. Any file that starts with the prefix will be ignored as if it
     * is not on disk.
     */
    private val ignoredPathPrefixes = setOf(
        // See: https://github.com/guardian/coverdrop/issues/1309 --- this file is a caching file
        // from `androidx.profileinstaller.ProfileVerifier`.
        "files/profileInstalled",
    )

    /**
     * All expected directories. We use this for the storage which will look different whether we
     * use Sloth or not.
     */
    private val expectedDirectories = setOf(
        "files/coverdrop/live/encrypted_storage/",
    )

    /**
     * All expected files. If they are not found, the test will fail.
     */
    private val expectedFiles = setOf(
        "files/coverdrop/live/dead_drops_v1.json",
        "files/coverdrop/live/private_sending_queue_v2.blob",
        "files/coverdrop/live/published_keys_v1.json",
        "files/coverdrop/live/status_event_v1.json",
        "shared_prefs/coverdrop_shared_prefs.xml",
    )

    /**
     * Files that set group access permissions by default (e.g. SharedPreferences)
     */
    private val ignoreGroupPermissions = setOf(
        "shared_prefs/coverdrop_shared_prefs.xml",
    )

    @Test
    fun whenAppInit_thenOnlyKnownDirectoriesAndFilesCreated_andCorrectPermissions() {
        // reset app state
        val dataDir = targetContext.dataDir
        dataDir.deleteRecursively()

        // assert we start with an empty data dir
        val allFilesBefore = dataDir.list()
        assertThat(allFilesBefore).isEmpty()

        internalInitializeCoverDropSynchronously()

        // record all files that exist while filtering out ignored folders
        val actualFiles = dataDir.walkTopDown()
            .filterNot { it.isDirectory }
            .map { it.relativeTo(dataDir).toString() }
            .filterNot { ignoredPathPrefixes.any { ignoredPath -> it.startsWith(ignoredPath) } }
            .toSet()

        // collect all expected directories and all expected individual files
        val foundExpectedFiles = emptySet<String>().toMutableSet()
        foundExpectedFiles += actualFiles.filter { actualFilePath ->
            expectedDirectories.any { expectedDirPattern ->
                actualFilePath.startsWith(expectedDirPattern)
            }
        }.toSet()
        foundExpectedFiles += actualFiles.filter { actualFilePath ->
            expectedFiles.any { expectedFilePath -> actualFilePath == expectedFilePath }
        }.toSet()

        // expected files should match observed files
        assertThat(actualFiles).containsExactlyElementsIn(foundExpectedFiles)

        // find any file that we were expecting but missing
        val missedFiles = expectedFiles - foundExpectedFiles
        assertThat(missedFiles).isEmpty()

        // get expected owner of our main directory
        val expectedOwner = dataDir.toPath().getOwner()!!.name

        // assert that all found files have the correct visibility
        for (existingFile in foundExpectedFiles) {
            val file = File(dataDir, existingFile)
            assertThat(file.exists()).isTrue()

            val path = file.toPath()

            // should all be owned by our application
            assertThat(path.getOwner()?.name).isEqualTo(expectedOwner)

            // should all be only read and writable by the owner
            val permissions = path.getPosixFilePermissions()
            Log.i("StorageAuditTest", "$path -> $permissions")

            assertThat(permissions).containsAtLeast(
                PosixFilePermission.OWNER_READ,
                PosixFilePermission.OWNER_WRITE
            )
            assertThat(permissions).containsNoneOf(
                PosixFilePermission.OTHERS_READ,
                PosixFilePermission.OTHERS_WRITE,
                PosixFilePermission.OTHERS_EXECUTE
            )
            if (!ignoreGroupPermissions.contains(existingFile)) {
                assertThat(permissions).containsNoneOf(
                    PosixFilePermission.GROUP_READ,
                    PosixFilePermission.GROUP_WRITE,
                    PosixFilePermission.GROUP_EXECUTE
                )
            }
        }
    }

    /**
     * List of all external file storage locations
     */
    private fun externalFilesDirTypes() = setOf(
        Environment.DIRECTORY_ALARMS,
        Environment.DIRECTORY_MOVIES,
        Environment.DIRECTORY_MUSIC,
        Environment.DIRECTORY_NOTIFICATIONS,
        Environment.DIRECTORY_PICTURES,
        Environment.DIRECTORY_PODCASTS,
        Environment.DIRECTORY_RINGTONES,
    )

    @Test
    fun whenAppInit_thenNothingIsWrittenToExternalStorage() {
        // reset app state
        val dataDir = targetContext.dataDir
        dataDir.deleteRecursively()

        // assert we start with an empty data dir
        val allFilesBefore = dataDir.list()
        assertThat(allFilesBefore).isEmpty()

        internalInitializeCoverDropSynchronously()

        // the public storage locations should be empty
        for (externalFilesDirType in externalFilesDirTypes()) {
            val externalFilesDir = targetContext.getExternalFilesDir(externalFilesDirType)
            assertThat(externalFilesDir?.listFiles()).isEmpty()
        }
    }

    private fun internalInitializeCoverDropSynchronously() {
        // We insert our test vectors to allow the test to run without internet access; otherwise
        // the `onAppInit` will block when run offline without previously cached data
        val instrumentationContext = getInstrumentation().context
        val testVectors = IntegrationTestVectors(
            context = instrumentationContext,
            scenario = TestScenario.Minimal
        )
        val configuration = CoverDropConfiguration(
            apiConfiguration = TestApiCallProvider.createTestApiConfiguration(),
            createApiCallProvider = { TestApiCallProvider(testVectors) },
            trustedOrgPks = testVectors.getKeys().getTrustedOrganisationKeys(),
            clock = TestClock(nowOverride = testVectors.getNow()),
            localTestMode = true,
        )

        CoverDropLib.onAppInit(
            applicationContext = targetContext.applicationContext as Application,
            configuration = configuration,
            coroutineScope = MainScope(),
            defaultDispatcher = Dispatchers.Default,
            exceptionHandler = CoverDropThrowingExceptionHandler(),
        )

        // wait for the fully initialized flag to become true
        runBlocking {
            withTimeout(60.seconds) {
                CoverDropLib.getInstance().getInitializationSuccessful().first { it }
            }
        }
    }
}
