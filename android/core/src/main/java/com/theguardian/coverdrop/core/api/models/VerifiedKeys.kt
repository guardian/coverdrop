package com.theguardian.coverdrop.core.api.models

import com.theguardian.coverdrop.core.crypto.PublicEncryptionKey
import com.theguardian.coverdrop.core.crypto.PublicSigningKey
import com.theguardian.coverdrop.core.models.JournalistId
import com.theguardian.coverdrop.core.utils.IClock
import com.theguardian.coverdrop.core.utils.filterValuesNotNull
import java.time.Instant

internal data class VerifiedKeys(
    val keys: List<VerifiedKeyHierarchy>,
)

internal data class VerifiedKeyHierarchy(
    val orgPk: TrustedRootSigningKey,
    val journalistsHierarchies: List<VerifiedJournalistsKeyHierarchy>,
    val coverNodeHierarchies: List<VerifiedCoverNodeKeyHierarchy>,
)

internal data class VerifiedCoverNodeKeyHierarchy(
    val provisioningPk: VerifiedSignedSigningKey,
    val coverNodes: VerifiedCoverNodeIdsAndKeys,
)

internal data class VerifiedJournalistsKeyHierarchy(
    val provisioningPk: VerifiedSignedSigningKey,
    val journalists: VerifiedJournalistIdsAndKeys,
)

internal typealias CoverNodeId = String
internal typealias VerifiedCoverNodeIdsAndKeys = Map<CoverNodeId, List<VerifiedKeyFamily>>

internal typealias VerifiedJournalistIdsAndKeys = Map<JournalistId, List<VerifiedKeyFamily>>

//
// Key types
//

internal data class TrustedRootSigningKey(
    val pk: PublicSigningKey,
)

internal data class VerifiedSignedSigningKey(
    val pk: PublicSigningKey,
)

internal data class VerifiedSignedEncryptionKey(
    val pk: PublicEncryptionKey,
    val notValidAfter: Instant,
)

internal class VerifiedKeyFamily(
    val idPk: VerifiedSignedSigningKey,
    val msgPks: List<VerifiedSignedEncryptionKey>,
)

//
// Convenience methods for iterating over keys in lower levels of the hierarchies
//

internal fun List<VerifiedJournalistsKeyHierarchy>.allJournalistToMessagingKeys(): Map<JournalistId, List<VerifiedSignedEncryptionKey>> {
    val map = HashMap<JournalistId, MutableList<VerifiedSignedEncryptionKey>>()
    for (hierarchy in this) {
        for ((journalistId, keyFamilies) in hierarchy.journalists) {
            for (keyFamily in keyFamilies) {
                for (key in keyFamily.msgPks) {
                    if (!map.containsKey(journalistId)) {
                        map[journalistId] = mutableListOf()
                    }
                    map[journalistId]!!.add(key)
                }
            }
        }
    }
    return map
}

internal fun List<VerifiedCoverNodeKeyHierarchy>.allCoverNodeToMessagingKeys(): Map<CoverNodeId, List<VerifiedSignedEncryptionKey>> {
    val map = HashMap<CoverNodeId, MutableList<VerifiedSignedEncryptionKey>>()
    for (hierarchy in this) {
        for ((coverNodeId, keyFamilies) in hierarchy.coverNodes) {
            for (keyFamily in keyFamilies) {
                for (key in keyFamily.msgPks) {
                    if (!map.containsKey(coverNodeId)) {
                        map[coverNodeId] = mutableListOf()
                    }
                    map[coverNodeId]!!.add(key)
                }
            }
        }
    }
    return map
}

internal fun VerifiedKeys.mostRecentMessagingKeyForJournalist(
    journalistId: JournalistId,
    clock: IClock,
): VerifiedSignedEncryptionKey {
    val messagingKeysForJournalists =
        this.keys.flatMap { it.journalistsHierarchies }.allJournalistToMessagingKeys()
    val maybeJournalistMsgKey = messagingKeysForJournalists[journalistId]
        ?.filter { it.notValidAfter.isAfter(clock.now()) }
        ?.maxByOrNull { it.notValidAfter }
    return checkNotNull(maybeJournalistMsgKey) {
        "No valid journalist message key candidate found for $journalistId"
    }
}

internal fun VerifiedKeys.mostRecentMessagingKeyForEachCoverNode(
    clock: IClock,
): Map<CoverNodeId, VerifiedSignedEncryptionKey> {
    val allCoverNodeMessagingKeys =
        this.keys.flatMap { it.coverNodeHierarchies }.allCoverNodeToMessagingKeys()
    return allCoverNodeMessagingKeys.map { entry ->
        entry.key to entry.value
            .filter { it.notValidAfter.isAfter(clock.now()) }
            .maxByOrNull { it.notValidAfter }
    }.toMap().filterValuesNotNull().toMap()
}

internal fun List<VerifiedCoverNodeKeyHierarchy>.allCoverNodeSigningKeys(): List<VerifiedSignedSigningKey> {
    return this.flatMap { hierarchy -> hierarchy.coverNodes.values }
        .flatMap { keyFamilies -> keyFamilies.map { keyFamily -> keyFamily.idPk } }
}
