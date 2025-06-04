package com.theguardian.coverdrop.ui.tests

import android.app.Application
import com.theguardian.coverdrop.core.ICoverDropLib
import dagger.hilt.android.HiltAndroidApp
import javax.inject.Inject

@HiltAndroidApp
class CoverDropApplication : Application() {

    @Inject
    lateinit var coverdrop: ICoverDropLib
}
