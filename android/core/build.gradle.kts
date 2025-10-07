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
    implementation("com.goterl:lazysodium-android:5.2.0@aar")
    implementation("net.java.dev.jna:jna:5.18.1@aar")

    testImplementation(libs.junit)

    androidTestImplementation(project(":testutils"))
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.androidx.work.testing)
    androidTestImplementation(libs.truth)
}

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = libs.versions.libraryGroupId.get()
            artifactId = "core"
            version = libs.versions.libraryVersion.get()

            pom {
                name.set("CoverDrop :core Android")
                description.set("The CoverDrop core library for Android")
                url.set("https://github.com/guardian/coverdrop")
                packaging = "aar"
                licenses {
                    license {
                        name.set("Apache License, Version 2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0.txt")
                    }
                }
                developers {
                    developer {
                        name.set("The Guardian's Secure Messaging Team")
                        email.set("secure.messaging@guardian.co.uk")
                        url.set("https://github.com/guardian/coverdrop")
                    }
                }
                organization {
                    name.set("Guardian News & Media")
                    url.set("https://www.theguardian.com")
                }
                scm {
                    connection.set("scm:git:github.com/guardian/coverdrop.git")
                    developerConnection.set("scm:git:ssh://github.com/guardian/coverdrop.git")
                    url.set("https://github.com/guardian/coverdrop/tree/main/android")
                }
            }

            // Use the artifacts called "release" for publishing.
            afterEvaluate {
                from(components["release"])
            }
        }

        repositories {
            maven {
                url = uri(layout.buildDirectory.dir("repo"))
            }
        }
    }
}

signing {
    if (!System.getenv("AUTOMATED_MAVEN_RELEASE_PGP_SECRET").isNullOrEmpty()) {
        // If the PGP secret is set, we use in-memory keys for signing.
        // This is useful for automated releases.
        useInMemoryPgpKeys(
            System.getenv("AUTOMATED_MAVEN_RELEASE_PGP_SECRET"),
            // We use a passwordless key so the an empty string is used as password here.
            "",
        )
    } else {
        // Otherwise, we use GPG command line tool for signing.
        useGpgCmd()
    }
    sign(publishing.publications)
}
