package com.theguardian.coverdrop.core

/**
 * Handler for exceptions thrown within the CoverDrop coroutine scope.
 */
interface ICoverDropExceptionHandler {
    operator fun invoke(e: Exception)
}

/**
 * Handler for exceptions that throws them. This will typically result in the app crashing.
 */
class CoverDropThrowingExceptionHandler : ICoverDropExceptionHandler {
    override fun invoke(e: Exception) {
        throw e
    }
}
