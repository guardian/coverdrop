import Foundation

actor DeadDropIdRepository {
    let file = CoverDropFiles.deadDropId

    func load() async throws -> DeadDropId {
        let data = try StorageManager.shared.readFile(file: file)
        return try JSONDecoder().decode(DeadDropId.self, from: Data(data))
    }

    func save(deadDropId: DeadDropId) async throws {
        let encodedData = try JSONEncoder().encode(deadDropId)
        try StorageManager.shared.writeFile(file: file, data: Array(encodedData))
    }
}
