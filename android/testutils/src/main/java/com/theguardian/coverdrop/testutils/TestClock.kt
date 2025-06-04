package com.theguardian.coverdrop.testutils

import com.theguardian.coverdrop.core.utils.IClock
import java.time.Duration
import java.time.Instant

class TestClock(private var nowOverride: Instant) : IClock {
    override fun now() = nowOverride

    fun advance(duration: Duration?) {
        nowOverride += duration
    }

    fun setNow(nowOverride: Instant) {
        this.nowOverride = nowOverride
    }
}
