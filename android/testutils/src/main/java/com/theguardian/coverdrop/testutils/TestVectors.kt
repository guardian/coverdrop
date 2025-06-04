package com.theguardian.coverdrop.testutils

import android.content.Context
import java.io.File

open class TestVectors(private val context: Context, basePath: String) {

    private val basePathFile = File(File("crypto-test-vectors"), basePath)

    fun readFile(path: String): ByteArray {
        val pathFile = File(basePathFile, path)
        return context.assets.open(pathFile.path).readBytes()
    }

    fun readJson(path: String) = readString(path)

    fun readString(path: String) = readFile(path).decodeToString().trim()
}
