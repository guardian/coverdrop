package com.theguardian.coverdrop.core.api.models

import androidx.annotation.Keep
import com.google.gson.annotations.SerializedName
import java.time.Instant

internal typealias CoverNodeIdentity = String
internal typealias JournalistIdentity = String

/**
 * The full response from the `public-keys` endpoint. It is divided into main parts. The
 * `journalistProfiles` contain the profiles of all journalists that are part of the organisation.
 * The `keys` contain the full key hierarchy and it references back into the profile using the
 * respective `journalistId`.
 */
@Keep // required to survive R8
data class PublishedKeysAndProfiles(
    @SerializedName("journalist_profiles")
    val journalistProfiles: List<PublishedJournalistProfile>,

    @SerializedName("default_journalist_id")
    val defaultJournalistId: String?,

    @SerializedName("keys")
    val keys: List<PublishedKeyHierarchy>,
)

@Keep // required to survive R8
data class PublishedKeyHierarchy(
    @SerializedName("org_pk")
    val orgPk: PublishedSignedSigningKey,

    @SerializedName("covernodes")
    val coverNodesKeyHierarchy: List<PublishedCoverNodeKeyHierarchy>,

    @SerializedName("journalists")
    val journalistsKeyHierarchy: List<PublishedJournalistsKeyHierarchy>,
)

@Keep // required to survive R8
data class PublishedCoverNodeKeyHierarchy(
    @SerializedName("provisioning_pk")
    val provisioningPk: PublishedSignedSigningKey,

    @SerializedName("covernodes")
    val coverNodes: HashMap<CoverNodeIdentity, List<PublishedKeyFamily>>,
)

@Keep // required to survive R8
data class PublishedJournalistsKeyHierarchy(
    @SerializedName("provisioning_pk")
    val provisioningPk: PublishedSignedSigningKey,

    @SerializedName("journalists")
    val journalists: HashMap<JournalistIdentity, List<PublishedKeyFamily>>,
)

@Keep // required to survive R8
data class PublishedJournalistProfile(
    @SerializedName("id")
    val id: String,

    @SerializedName("display_name")
    val displayName: String,

    @SerializedName("sort_name")
    val sortName: String,

    @SerializedName("description")
    val description: String,

    @SerializedName("is_desk")
    val isDesk: Boolean,

    @SerializedName("tag")
    val tag: String,

    @SerializedName("status")
    val status: PublishedJournalistVisibility,
)

@Keep // required to survive R8
enum class PublishedJournalistVisibility {
    @SerializedName("HIDDEN")
    VISIBLE,

    @SerializedName("HIDDEN_FROM_UI")
    HIDDEN_FROM_UI,

    @SerializedName("HIDDEN_FROM_RESPONSE")
    HIDDEN_FROM_RESPONSE, // for completeness, we should never see these
}

//
// Key types
//

@Keep // required to survive R8
data class PublishedSignedSigningKey(
    @SerializedName("key")
    val key: String,

    @SerializedName("certificate")
    val certificate: String,

    @SerializedName("not_valid_after")
    val notValidAfter: Instant,
)

@Keep // required to survive R8
data class PublishedSignedEncryptionKey(
    @SerializedName("key")
    val key: String,

    @SerializedName("certificate")
    val certificate: String,

    @SerializedName("not_valid_after")
    val notValidAfter: Instant,
)

@Keep // required to survive R8
data class PublishedKeyFamily(
    @SerializedName("id_pk")
    val idPk: PublishedSignedSigningKey,

    @SerializedName("msg_pks")
    val msgPks: List<PublishedSignedEncryptionKey>,
)

@Keep // required to survive R8
data class PublicKey(
    @SerializedName("key")
    val key: String,
)
