// Some libraries pulled in by AGP and lint have known vulnerabilities and newer AGP versions do
// not fix them yet. We raise them to patched versions here, so that we can keep the AGP version in
// sync with the main app. Remove these lines once AGP ships the patched versions by itself.
buildscript {
    dependencies {
        constraints {
            classpath("org.jdom:jdom2:2.0.6.1")
            classpath("org.bitbucket.b_c:jose4j:0.9.6")
            classpath("org.bouncycastle:bcprov-jdk18on:1.85")
            classpath("org.bouncycastle:bcpkix-jdk18on:1.85")
            classpath("org.bouncycastle:bcutil-jdk18on:1.85")
            classpath("com.google.guava:guava:33.7.2-jre")
        }
    }
}

plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.kotlin.compose) apply false
    alias(libs.plugins.hilt) apply false
    alias(libs.plugins.ksp) apply false
    alias(libs.plugins.nexus.publish)
}


group = libs.versions.libraryGroupId.get()
version = libs.versions.libraryVersion.get()

// The vulnerable libraries above also show up within the modules, e.g. via the lint tool (Bouncy
// Castle) or Truth and annotation processors (Guava). This only affects how our own builds resolve
// dependencies and does not change the published `core` and `ui` libraries.
subprojects {
    configurations.configureEach {
        resolutionStrategy.eachDependency {
            if (requested.group == "org.bouncycastle" && requested.name.endsWith("-jdk18on")) {
                useVersion("1.85")
            }
            if (requested.group == "com.google.guava" && requested.name == "guava") {
                // Guava comes in an `-android` and a `-jre` flavour; keep whichever was requested
                val flavour = if (requested.version.orEmpty().endsWith("-android")) "android" else "jre"
                useVersion("33.7.2-$flavour")
            }
        }
    }
}

nexusPublishing {
    repositories {
        sonatype {
            nexusUrl.set(uri("https://ossrh-staging-api.central.sonatype.com/service/local/"))
            snapshotRepositoryUrl.set(uri("https://central.sonatype.com/repository/maven-snapshots/"))
            // Sonatype token provides username and passwords as revokable secrets combined with a
            // colon. We split them and provide it to the nexus plugin. See here for more:
            // https://github.com/guardian/gha-scala-library-release-workflow/commit/23a148a03cf71bb2093a91f047d3c368adcdf45c
            val token = System.getenv("AUTOMATED_MAVEN_RELEASE_SONATYPE_TOKEN") ?: ":"
            val (uname, pwd) = token.split(":")
            username = uname
            password = pwd
        }
    }
}
