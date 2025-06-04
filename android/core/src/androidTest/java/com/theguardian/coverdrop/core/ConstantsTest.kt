package com.theguardian.coverdrop.core

import com.google.common.truth.Truth.assertThat
import com.theguardian.coverdrop.core.generated.MESSAGE_PADDING_LEN
import com.theguardian.coverdrop.core.generated.USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN
import org.junit.Test


class ConstantsTest {

    @Test
    fun check_constants_present() {
        assertThat(USER_TO_COVERNODE_ENCRYPTED_MESSAGE_LEN).isGreaterThan(MESSAGE_PADDING_LEN)
    }
}
