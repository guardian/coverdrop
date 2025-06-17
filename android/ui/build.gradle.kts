import org.jetbrains.kotlin.konan.properties.loadProperties

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.jetbrains.kotlin.android)
    alias(libs.plugins.kotlin.compose)
    id("kotlin-kapt")
    alias(libs.plugins.hilt)
    id("signing")
    id("maven-publish")
}

apply(from = "../android-force-deps-upgrades.gradle")
apply(from = "../android-lint.gradle")

val prodConfig = loadProperties("coverdrop-prod.properties")

android {
    namespace = "com.theguardian.coverdrop.ui"
    compileSdk = libs.versions.compileSdk.get().toInt()

    defaultConfig {
        minSdk = libs.versions.minSdk.get().toInt()
        targetSdk = libs.versions.targetSdk.get().toInt()

        vectorDrawables {
            useSupportLibrary = true
        }

        // These values are used in the integration tests. Running them against the production environment
        // is useful for the CI tests as we do not need to spin up local services.
        buildConfigField("String", "API_BASE_URL", prodConfig.getProperty("coverdrop.apiBaseUrl"))
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

    buildTypes {
        named("release") {
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

    publishing {
        singleVariant("release") {
            withSourcesJar()

            // Disabled while Dokka is broken for Java 17: https://github.com/Kotlin/dokka/issues/2956
            // this should come with little downsides, as we ship the sources jar
            // withJavadocJar()
        }
    }
}

dependencies {
    implementation(project(":core"))
    implementation(platform(libs.androidx.compose.bom))

    kapt(libs.hilt.compiler)

    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.compose.material)
    implementation(libs.androidx.compose.runtimeLiveData)
    implementation(libs.androidx.compose.uiTooling)
    implementation(libs.androidx.constraintlayout.compose)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.hilt.navigation.compose)
    implementation(libs.androidx.material.icons.core)
    implementation(libs.androidx.material.icons.extended)
    implementation(libs.androidx.lifecycle.livedata.ktx)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.lifecycle.viewmodel.compose)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.accompanist.pager.indicators)
    implementation(libs.accompanist.pager)
    implementation(libs.hilt.android)

    debugImplementation(libs.androidx.compose.uiToolingPreview)
}

kapt {
    correctErrorTypes = true
}

publishing {
    publications {
        register<MavenPublication>("release") {
            groupId = libs.versions.libraryGroupId.get()
            artifactId = "ui"
            version = libs.versions.libraryVersion.get()

            pom {
                name.set("CoverDrop :ui Android")
                description.set("The CoverDrop UI library for Android")
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
                url = uri("$buildDir/repo")
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
