plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.jetbrains.kotlin.android)
    `maven-publish`
    signing
}

apply(from = "../android-force-deps-upgrades.gradle")

android {
    namespace = "com.theguardian.coverdrop.core"
    compileSdk = libs.versions.compileSdk.get().toInt()

    defaultConfig {
        minSdk = libs.versions.minSdk.get().toInt()
        targetSdk = libs.versions.targetSdk.get().toInt()

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
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

    publishing {
        singleVariant("release") {
            withSourcesJar()

            // Disabled while Dokka is broken for Java 17: https://github.com/Kotlin/dokka/issues/2956
            // this should come with little downsides, as we ship the sources jar
            // withJavadocJar()
        }
    }
    buildFeatures {
        buildConfig = true
    }
}

dependencies {
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.appcompat)
    implementation(libs.gson)
    implementation(libs.androidx.work.runtime.ktx)
    implementation(libs.com.scottyab.rootbeer)
    implementation(libs.com.lambdapioneer.sloth)

    // lib-sodium dependencies; the "@aar" is important
    implementation("com.goterl:lazysodium-android:5.0.2@aar")
    implementation("net.java.dev.jna:jna:5.8.0@aar")

    testImplementation(libs.junit)

    androidTestImplementation(project(":testutils"))
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.androidx.work.testing)
    androidTestImplementation(libs.truth)
}

afterEvaluate {
    publishing {
        publications {
            create<MavenPublication>("release") {
                from(components["release"])
                groupId = "com.theguardian.coverdrop"
                artifactId = "core"
                version = "0.0.1"
            }
        }
        repositories {
            maven {
                url = uri("$buildDir/repo")
            }
        }
    }
}

signing {
    useGpgCmd()
    sign(publishing.publications)
}
