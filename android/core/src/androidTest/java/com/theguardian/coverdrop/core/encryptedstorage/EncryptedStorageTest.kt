package com.theguardian.coverdrop.core.encryptedstorage

import android.content.Context
import androidx.test.platform.app.InstrumentationRegistry
import com.google.common.truth.Truth.assertThat
import com.lambdapioneer.sloth.SlothLib
import com.theguardian.coverdrop.core.createLibSodium
import com.theguardian.coverdrop.core.crypto.PassphraseWordList
import com.theguardian.coverdrop.core.persistence.CoverDropFileManager
import com.theguardian.coverdrop.core.persistence.CoverDropFiles
import com.theguardian.coverdrop.core.persistence.CoverDropNamespace
import com.theguardian.coverdrop.core.persistence.MailboxContent
import com.theguardian.coverdrop.core.utils.DefaultClock
import com.theguardian.coverdrop.core.utils.expectThrows
import com.theguardian.coverdrop.testutils.TestClock
import org.junit.Assume.assumeNoException
import org.junit.Before
import org.junit.Test
import java.time.Instant

internal class EncryptedStorageWithPasswordTest : EncryptedStorageTest() {
    val context: Context = InstrumentationRegistry.getInstrumentation().targetContext

    private val fileManager = CoverDropFileManager(context, DefaultClock(), CoverDropNamespace.TEST)
    private val libSodium = createLibSodium()
    private val passphraseWordList = PassphraseWordList.createFromEffWordList(context)

    private val encryptedStorage = EncryptedStorageWithPassword(
        libSodium = libSodium,
        encryptedStorageConfiguration = selectEncryptedStorageConfiguration(
            useSecureElement = false,
            isForAutomatedTest = true
        ) as EncryptedStorageConfiguration.PasswordOnly,
        fileManager = fileManager,
        passphraseWordList = passphraseWordList,
    )

    override fun getInstance() = encryptedStorage

    @Before
    fun setUp() = super.internal_setUp(fileManager)

    @Test
    fun test_onAppStart_whenCalled_thenFilesExists() =
        internal_test_onAppStart_whenCalled_thenFilesExists(fileManager)

    @Test
    fun test_whenOnAppStartCalledAgain_thenLastModifiedStampsUpdated() =
        internal_test_whenOnAppStartCalledAgain_thenLastModifiedStampsUpdated(fileManager)

    @Test
    fun test_whenOnCreateOrResetStorageCalled_thenSessionThatAllowsReading() =
        internal_test_whenOnCreateOrResetStorageCalled_thenSessionThatAllowsReading()

    @Test
    fun test_whenCreateAndWriteAndUnlock_thenContentMatches() =
        internal_test_whenCreateAndWriteAndUnlock_thenContentMatches()

    @Test
    fun test_whenUnlockAndWriteAndRead_thenContentMatches() =
        internal_test_whenUnlockAndWriteAndRead_thenContentMatches()

    @Test
    fun test_whenReadWithDifferentPassphrase_thenFails() =
        internal_test_whenReadWithDifferentPassphrase_thenFails()

    @Test
    fun test_whenResetBetweenWriteAndRead_thenFails() =
        internal_test_whenResetBetweenWriteAndRead_thenFails()
}

internal class EncryptedStorageWithSecureElementTest : EncryptedStorageTest() {
    val context: Context = InstrumentationRegistry.getInstrumentation().targetContext

    private val namespace = CoverDropNamespace.TEST
    private val fileManager = CoverDropFileManager(context, clock = DefaultClock(), namespace)
    private val passphraseWordList = PassphraseWordList.createFromEffWordList(context)

    override fun getInstance(): IEncryptedStorage {
        val encryptedStorageConfiguration = selectEncryptedStorageConfiguration(
            useSecureElement = true,
            isForAutomatedTest = true
        ) as EncryptedStorageConfiguration.SecureElement

        val slothLib = SlothLib(
            pwHash = Argon2PwHashBinding(
                libSodium = createLibSodium(),
                params = encryptedStorageConfiguration.secureElementParameters,
            )
        )

        // we skip the test if Sloth cannot be initialized; this is e.g. true for emulators
        try {
            slothLib.init(context)
        } catch (e: UnsupportedOperationException) {
            assumeNoException(
                "Skipped because the device does not support Sloth (e.g. no Secure Element)",
                e
            )
        }

        return EncryptedStorageWithSecureElement(
            context = context,
            slothLib = slothLib,
            encryptedStorageConfiguration = encryptedStorageConfiguration,
            fileManager = fileManager,
            namespace = namespace,
            passphraseWordList = passphraseWordList,
            libSodium = createLibSodium(),
        )
    }

    @Before
    fun setUp() = super.internal_setUp(fileManager)

    @Test
    fun test_onAppStart_whenCalled_thenFilesExists() =
        internal_test_onAppStart_whenCalled_thenFilesExists(fileManager)

    @Test
    fun test_whenOnAppStartCalledAgain_thenLastModifiedStampsUpdated() =
        internal_test_whenOnAppStartCalledAgain_thenLastModifiedStampsUpdated(fileManager)

    @Test
    fun test_whenOnCreateOrResetStorageCalled_thenSessionThatAllowsReading() =
        internal_test_whenOnCreateOrResetStorageCalled_thenSessionThatAllowsReading()

    @Test
    fun test_whenCreateAndWriteAndUnlock_thenContentMatches() =
        internal_test_whenCreateAndWriteAndUnlock_thenContentMatches()

    @Test
    fun test_whenUnlockAndWriteAndRead_thenContentMatches() =
        internal_test_whenUnlockAndWriteAndRead_thenContentMatches()

    @Test
    fun test_whenReadWithDifferentPassphrase_thenFails() =
        internal_test_whenReadWithDifferentPassphrase_thenFails()

    @Test
    fun test_whenResetBetweenWriteAndRead_thenFails() =
        internal_test_whenResetBetweenWriteAndRead_thenFails()
}

internal abstract class EncryptedStorageTest {

    abstract fun getInstance(): IEncryptedStorage

    fun internal_setUp(fileManager: CoverDropFileManager) {
        fileManager.delete(CoverDropFiles.ENCRYPTED_STORAGE_DIRECTORY)
        fileManager.initialize()
    }

    fun internal_test_onAppStart_whenCalled_thenFilesExists(fileManager: CoverDropFileManager) {
        val instance = getInstance()
        val encryptedStoragePath = fileManager.path(CoverDropFiles.ENCRYPTED_STORAGE_DIRECTORY)

        // the directory exists, but there are no files
        assertThat(encryptedStoragePath.exists()).isEqualTo(true)
        assertThat(encryptedStoragePath.list()).isEmpty()

        instance.onAppStart()

        // the directory exists and there are files
        assertThat(encryptedStoragePath.exists()).isEqualTo(true)
        assertThat(encryptedStoragePath.list()).isNotEmpty()
    }

    fun internal_test_whenOnAppStartCalledAgain_thenLastModifiedStampsUpdated(fileManager: CoverDropFileManager) {
        val instance = getInstance()
        val encryptedStoragePath = fileManager.path(CoverDropFiles.ENCRYPTED_STORAGE_DIRECTORY)

        // create initially
        instance.onAppStart()

        // collect the last modified timestamps for all files in the folder
        val lastModifiedMap1 = encryptedStoragePath.walkBottomUp()
            .filter { it != encryptedStoragePath } // ignore the root folder itself
            .map { Pair(it, it.lastModified()) }
            .toMap()

        // restart app after at least one second wait
        Thread.sleep(1500)
        instance.onAppStart()

        // again collect the last modified timestamps for all files in the folder
        val lastModifiedMap2 = encryptedStoragePath.walkBottomUp()
            .filter { it != encryptedStoragePath } // ignore the root folder itself
            .map { Pair(it, it.lastModified()) }
            .toMap()

        // ensure we actually captured the same files
        assertThat(lastModifiedMap1.keys).containsExactlyElementsIn(lastModifiedMap2.keys)

        // the last modified timestamp should be larger
        for (kv in lastModifiedMap1.entries) {
            assertThat(lastModifiedMap2[kv.key]).isGreaterThan(kv.value)
        }
    }

    fun internal_test_whenOnCreateOrResetStorageCalled_thenSessionThatAllowsReading() {
        val instance = getInstance()
        instance.onAppStart()

        val passphrase = instance.generateNewRandomPassphrase()
        val session = instance.createOrResetStorage(passphrase)
        assertThat(session).isNotNull()

        val actual = instance.loadFromStorage(session)
        assertThat(actual).isNotNull()
    }

    // writing with the active session, reading with the unlocked session
    fun internal_test_whenCreateAndWriteAndUnlock_thenContentMatches() {
        val instance = getInstance()
        instance.onAppStart()

        val passphrase = instance.generateNewRandomPassphrase()
        val content = createMailboxContent()

        // write with the session from creating
        instance.createOrResetStorage(passphrase).also { session ->
            instance.saveToStorage(session, content)
        }

        // read with the session from unlocking
        instance.unlockSession(passphrase).also { session ->
            val readContent = instance.loadFromStorage(session)
            assertThat(readContent).isEqualTo(content)
        }
    }

    // writing with the unlocked session, reading with the unlocked session
    fun internal_test_whenUnlockAndWriteAndRead_thenContentMatches() {
        val instance = getInstance()
        instance.onAppStart()

        val passphrase = instance.generateNewRandomPassphrase()
        instance.createOrResetStorage(passphrase)

        val content = createMailboxContent()

        // write with the session from unlocking
        instance.unlockSession(passphrase).also { session ->
            instance.saveToStorage(session, content)
        }

        // read with the session from unlocking
        instance.unlockSession(passphrase).also { session ->
            val readContent = instance.loadFromStorage(session)
            assertThat(readContent).isEqualTo(content)
        }
    }

    fun internal_test_whenReadWithDifferentPassphrase_thenFails() {
        val instance = getInstance()
        instance.onAppStart()

        val passphrase1 = instance.generateNewRandomPassphrase()
        val content = createMailboxContent()

        // write with first passphrase
        instance.createOrResetStorage(passphrase1).also { session ->
            instance.saveToStorage(session, content)
        }

        val passphrase2 = instance.generateNewRandomPassphrase()

        // read with second passphrase
        instance.unlockSession(passphrase2).also { session ->
            expectThrows(EncryptedStorageAuthenticationFailed::class.java) {
                instance.loadFromStorage(session)
            }
        }
    }

    fun internal_test_whenResetBetweenWriteAndRead_thenFails() {
        val instance = getInstance()
        instance.onAppStart()

        val passphrase = instance.generateNewRandomPassphrase()
        instance.createOrResetStorage(passphrase)

        val content = createMailboxContent()

        // write with the session from unlocking
        instance.unlockSession(passphrase).also { session ->
            instance.saveToStorage(session, content)
        }

        // reset storage
        instance.createOrResetStorage(instance.generateNewRandomPassphrase())

        // read with the session from unlocking
        instance.unlockSession(passphrase).also { session ->
            expectThrows(EncryptedStorageAuthenticationFailed::class.java) {
                instance.loadFromStorage(session)
            }
        }
    }

    // The content is always different due to the newly created encryption key pairs
    private fun createMailboxContent() = MailboxContent.newEmptyMailbox(createLibSodium())
}
