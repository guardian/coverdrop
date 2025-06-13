import org.jetbrains.kotlin.konan.properties.loadProperties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.jetbrains.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    id("kotlin-kapt")
    alias(libs.plugins.hilt)
}

apply(from = "../android-force-deps-upgrades.gradle")
apply(from = "../android-lint.gradle")

// The endpoint URLs and trusted org keys for the PROD stack
val prodConfig = loadProperties("coverdrop-prod.properties")

// The endpoint URLs and trusted org keys for the STAGING stack
val stagingConfig = loadProperties("coverdrop-staging.properties")

android {
    namespace = "com.theguardian.coverdrop"
    compileSdk = libs.versions.compileSdk.get().toInt()

    defaultConfig {
        minSdk = libs.versions.minSdk.get().toInt()
        targetSdk = libs.versions.targetSdk.get().toInt()

        applicationId = "com.theguardian.coverdrop"
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "com.theguardian.coverdrop.audit.runner.HiltTestRunner"

        vectorDrawables {
            useSupportLibrary = true
        }
    }

    splits {
        abi {
            isEnable = true
            reset()
            include("armeabi-v7a", "arm64-v8a", "x86", "x86_64")
            isUniversalApk = true
        }
    }

    buildTypes {
        // Variant that is closely mimicking the production version of the main app (points to PROD)
        named("release") {
            isDebuggable = false
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("debug")

            buildConfigField("Boolean", "LOCAL_TEST_MODE_ENABLED", "false")
            buildConfigField("Boolean", "SCREEN_CAPTURE_PROTECTION_DISABLED", "false")

            buildConfigField(
                "String",
                "API_BASE_URL",
                prodConfig.getProperty("coverdrop.apiBaseUrl")
            )
            buildConfigField(
                "String",
                "MESSAGING_BASE_URL",
                prodConfig.getProperty("coverdrop.fastlyBaseUrl")
            )
            buildConfigField(
                "String",
                "TRUSTED_ORG_PKS",
                prodConfig.getProperty("coverdrop.trustedOrgPks")
            )
        }

        // A release variant that has certain security features disable to simplify testing (points to PROD)
        // That's a version we can hand-over to UX testing or copy writers
        register("ux") {
            initWith(getByName("release"))
            matchingFallbacks += listOf("release")

            buildConfigField("Boolean", "SCREEN_CAPTURE_PROTECTION_DISABLED", "true")
        }

        // A UX variant that points to STAGING
        register("uxstaging") {
            initWith(getByName("release"))
            matchingFallbacks += listOf("release")

            buildConfigField("Boolean", "SCREEN_CAPTURE_PROTECTION_DISABLED", "true")

            buildConfigField(
                "String",
                "API_BASE_URL",
                stagingConfig.getProperty("coverdrop.apiBaseUrl")
            )
            buildConfigField(
                "String",
                "MESSAGING_BASE_URL",
                stagingConfig.getProperty("coverdrop.fastlyBaseUrl")
            )
            buildConfigField(
                "String",
                "TRUSTED_ORG_PKS",
                stagingConfig.getProperty("coverdrop.trustedOrgPks")
            )
        }

        // The default variant for local development (points to STAGING)
        named("debug") {
            isDebuggable = true

            buildConfigField("Boolean", "LOCAL_TEST_MODE_ENABLED", "true")
            buildConfigField("Boolean", "SCREEN_CAPTURE_PROTECTION_DISABLED", "true")

            buildConfigField(
                "String",
                "API_BASE_URL",
                stagingConfig.getProperty("coverdrop.apiBaseUrl")
            )
            buildConfigField(
                "String",
                "MESSAGING_BASE_URL",
                stagingConfig.getProperty("coverdrop.fastlyBaseUrl")
            )
            buildConfigField(
                "String",
                "TRUSTED_ORG_PKS",
                stagingConfig.getProperty("coverdrop.trustedOrgPks")
            )
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
    // Change the in the following two lines the expression `project(":$1")` to
    // `"com.theguardian.coverdrop:$1:0.0.1"` to build against the local maven artefacts.
    // This can also be done by running `scripts/patch_app_to_use_maven.sh`.
    implementation(project(":core"))
    implementation(project(":ui"))
    implementation(platform(libs.androidx.compose.bom))

    kapt(libs.hilt.compiler)
    kaptAndroidTest(libs.hilt.android.compiler)

    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.compose.material)
    implementation(libs.androidx.compose.runtimeLiveData)
    implementation(libs.squareup.okhttp3.okhttp)
    implementation(libs.squareup.okhttp3.dnsoverhttps)
    implementation(libs.hilt.android)

    testImplementation(libs.junit)
    androidTestImplementation(project(":testutils"))
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.hilt.android.testing)
    androidTestImplementation(libs.truth)
}

kapt {
    correctErrorTypes = true
}
