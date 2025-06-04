package com.theguardian.coverdrop.core.crypto

import java.util.Vector


interface Signable {
    fun asBytes(): ByteArray
}

internal class SignableVector(
    bytes: ByteArray = byteArrayOf(),
) : Signable, Vector<Byte>(bytes.toList()) {
    override fun asBytes(): ByteArray = this.toByteArray()

    companion object {
        internal fun fromString(message: String) = SignableVector(message.toByteArray())
    }
}
