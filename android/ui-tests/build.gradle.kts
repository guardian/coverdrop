plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.jetbrains.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    id("kotlin-kapt")
    alias(libs.plugins.hilt)
}

apply(from = "../android-force-deps-upgrades.gradle")

android {
    namespace = "com.theguardian.coverdrop.ui.tests"
    compileSdk = libs.versions.compileSdk.get().toInt()

    defaultConfig {
        minSdk = libs.versions.minSdk.get().toInt()

        applicationId = "com.theguardian.coverdrop.ui.tests"
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "com.theguardian.coverdrop.ui.tests.runner.HiltTestRunner"

        vectorDrawables {
            useSupportLibrary = true
        }
    }

    buildTypes {
        named("release") {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("debug")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = libs.versions.java.get()
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    composeOptions {
        kotlinCompilerExtensionVersion = libs.versions.compose.compiler.get()
    }

    packaging {
        resources {
            excludes += "/META-INF/{AL2.0,LGPL2.1}"
        }
    }

    // work-around: https://issuetracker.google.com/issues/217593040#comment6
    kotlinOptions {
        freeCompilerArgs += listOf(
            "-Xjvm-default=all"
        )
    }
}

dependencies {
    implementation(project(":core"))
    implementation(project(":ui"))
    implementation(platform(libs.androidx.compose.bom))

    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.compose.runtimeLiveData)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.hilt.android)

    kapt(libs.hilt.compiler)
    kaptAndroidTest(libs.hilt.android.compiler)

    debugImplementation(libs.androidx.compose.uiTestManifest)
    debugImplementation(libs.androidx.compose.uiTooling)

    testImplementation(libs.junit)
    androidTestImplementation(project(":testutils"))
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.compose.uiTestJunit4)
    androidTestImplementation(libs.androidx.navigation.testing)
    androidTestImplementation(libs.androidx.runner)
    androidTestImplementation(libs.hilt.android.testing)
    androidTestImplementation(libs.truth)
}

kapt {
    correctErrorTypes = true
}
