plugins {
    alias(libs.plugins.android.application) apply false
    alias(libs.plugins.android.library) apply false
    alias(libs.plugins.jetbrains.kotlin.android) apply false
    alias(libs.plugins.kotlin.compose) apply false
    alias(libs.plugins.hilt) apply false
    alias(libs.plugins.nexus.publish)
}


group = libs.versions.libraryGroupId.get()
version = libs.versions.libraryVersion.get()

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
