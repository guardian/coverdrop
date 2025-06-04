package com.theguardian.coverdrop.core.utils

internal fun <K, V> Map<K, V?>.filterValuesNotNull(): Map<K, V> {
    return filterValues { it != null }.mapValues { it.value!! }
}
