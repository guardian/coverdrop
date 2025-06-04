package com.theguardian.coverdrop.core.crypto

import java.util.Vector


interface Encryptable {
    fun asUnencryptedBytes(): ByteArray
}

internal class EncryptableVector(
    bytes: ByteArray = byteArrayOf(),
) : Encryptable,
    Vector<Byte>(bytes.toList()) {
    override fun asUnencryptedBytes(): ByteArray = this.toByteArray()

    companion object {
        internal fun fromString(message: String) = EncryptableVector(message.toByteArray())
    }
}
