package com.theguardian.coverdrop.testutils

import android.content.Context
import java.io.File
import java.time.Instant

enum class TestScenario(val path: String) {
    Minimal("minimal_scenario"),
    MinimalLegacy("minimal_scenario_legacy"),
    SetSystemStatus("set_system_status"),
    Messaging("messaging_scenario"),
    KeyRotations("key_rotations"),
}

open class IntegrationTestVectors(
    private val context: Context,
    scenario: TestScenario,
) {
    private val basePathFile = File(File("integration-test-vectors"), scenario.path)

    fun readJson(folder: String, filename: String? = null) = readString(folder, filename)

    fun getKeys() = IntegrationTestKeys(context)

    fun getNow(filename: String? = null): Instant {
        return when (filename) {
            null -> getKeys().getNow()
            else -> Instant.parse(readString("timestamp", filename).trim('"'))
        }
    }

    private fun readFile(folder: String, filename: String? = null): ByteArray {
        // if no specific path is given, we return the first file in the folder (usually starting
        // with "001_")
        val folderPath = File(basePathFile, folder)
        val path = filename ?: listFiles(folderPath)?.first()!!

        val pathFile = File(folderPath, path)
        return context.assets.open(pathFile.path).readBytes()
    }

    private fun readString(folder: String, filename: String? = null): String {
        return readFile(folder, filename).decodeToString().trim()
    }

    private fun listFiles(folderPath: File): Array<out String>? {
        return context.assets.list(folderPath.path)
    }
}
