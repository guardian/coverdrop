## Backup Public Key Family

This describes how the backup protocol and key hierarchy work

More to come later 


This is the shape of the response for backup keys for the public-keys endpoint
```json
{
    "keys": [
        {
            "org_pk": {
                "key": "1037f9c40656adb2cc469e68758015c2d90f252c94eae10c4bf38c40d25f67b3",
                "certificate": "46df10d92a1be0bce3eb8208ce16c61f4f93883c51f86feba307e817f764361c0f129e4d213f5d3ffcf597a30a3893794bec67fb12fe17da43afc4345f686a01",
                "not_valid_after": "2026-03-04T09:37:36.830461067Z"
            },
            {
                "backups": [
                    {
                        "id_pk": {
                            "key": "36059c4c5be32635cc6eaf538b0703ae68b055a0c1dffb156d6d7ca1821328da",
                            "certificate": "8b204a690ad711cee20b81f372568a4c2f8f610b461fe731fe1cfabd3de8ae9e59f6aca1ce6cac57e4eebb9ed30320eccefcf47b914a20972db6b207749ceb0d",
                            "not_valid_after": "2025-10-09T17:31:47.509054799Z"
                        },
                        "msg_pks": [
                        {
                            "key": "6f1ed5697898a9470e41e4d5dd5e10bcf65a6cd045de66fa808768abf8a5153e",
                            "certificate": "877b68833d13785687850060cc3ed2589a68183d383f8a545992a34f27da1c0e4cf05a52c1bbe5887058c67793bf5e843bb7afd01bec4101084901acd9034a00",
                            "not_valid_after": "2025-10-03T00:42:45.724717606Z"
                        },
                    }
                ]
            }
        }
    ]
}


```
