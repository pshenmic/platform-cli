use dpp::platform_value::{platform_value, Value};

pub struct Constants;

impl Constants {
    pub fn dpns_data_contract_value() -> Value {
        platform_value!({
          "$format_version": "0",
          "id": "GWRSAVFMjXx8HpQFaNJMqBV7MBgMK4br5UESsB4S31Ec",
          "ownerId": "11111111111111111111111111111111",
          "version": 1,
          "documentSchemas": {
            "domain": {
              "documentsMutable": false,
              "canBeDeleted": true,
              "transferable": 1,
              "tradeMode": 1,
              "type": "object",
              "indices": [
                {
                  "name": "parentNameAndLabel",
                  "properties": [
                    {
                      "normalizedParentDomainName": "asc"
                    },
                    {
                      "normalizedLabel": "asc"
                    }
                  ],
                  "unique": true,
                  "contested": {
                    "fieldMatches": [
                      {
                        "field": "normalizedLabel",
                        "regexPattern": "^[a-zA-Z01-]{3,19}$"
                      }
                    ],
                    "resolution": 0,
                    "description": "If the normalized label part of this index is less than 20 characters (all alphabet a-z, A-Z, 0, 1, and -) then a masternode vote contest takes place to give out the name"
                  }
                },
                {
                  "name": "identityId",
                  "nullSearchable": false,
                  "properties": [
                    {
                      "records.identity": "asc"
                    }
                  ]
                }
              ],
              "properties": {
                "label": {
                  "type": "string",
                  "pattern": "^[a-zA-Z0-9][a-zA-Z0-9-]{0,61}[a-zA-Z0-9]$",
                  "minLength": 3,
                  "maxLength": 63,
                  "position": 0,
                  "description": "Domain label. e.g. 'Bob'."
                },
                "normalizedLabel": {
                  "type": "string",
                  "pattern": "^[a-hj-km-np-z0-9][a-hj-km-np-z0-9-]{0,61}[a-hj-km-np-z0-9]$",
                  "maxLength": 63,
                  "position": 1,
                  "description": "Domain label converted to lowercase for case-insensitive uniqueness validation. \"o\", \"i\" and \"l\" replaced with \"0\" and \"1\" to mitigate homograph attack. e.g. 'b0b'",
                  "$comment": "Must be equal to the label in lowercase. \"o\", \"i\" and \"l\" must be replaced with \"0\" and \"1\"."
                },
                "parentDomainName": {
                  "type": "string",
                  "pattern": "^$|^[a-zA-Z0-9][a-zA-Z0-9-]{0,61}[a-zA-Z0-9]$",
                  "minLength": 0,
                  "maxLength": 63,
                  "position": 2,
                  "description": "A full parent domain name. e.g. 'dash'."
                },
                "normalizedParentDomainName": {
                  "type": "string",
                  "pattern": "^$|^[a-hj-km-np-z0-9][a-hj-km-np-z0-9-\\.]{0,61}[a-hj-km-np-z0-9]$",
                  "minLength": 0,
                  "maxLength": 63,
                  "position": 3,
                  "description": "A parent domain name in lowercase for case-insensitive uniqueness validation. \"o\", \"i\" and \"l\" replaced with \"0\" and \"1\" to mitigate homograph attack. e.g. 'dash'",
                  "$comment": "Must either be equal to an existing domain or empty to create a top level domain. \"o\", \"i\" and \"l\" must be replaced with \"0\" and \"1\". Only the data contract owner can create top level domains."
                },
                "preorderSalt": {
                  "type": "array",
                  "byteArray": true,
                  "minItems": 32,
                  "maxItems": 32,
                  "position": 4,
                  "description": "Salt used in the preorder document"
                },
                "records": {
                  "type": "object",
                  "properties": {
                    "identity": {
                      "type": "array",
                      "byteArray": true,
                      "minItems": 32,
                      "maxItems": 32,
                      "position": 1,
                      "contentMediaType": "application/x.dash.dpp.identifier",
                      "description": "Identifier name record that refers to an Identity"
                    }
                  },
                  "minProperties": 1,
                  "position": 5,
                  "additionalProperties": false
                },
                "subdomainRules": {
                  "type": "object",
                  "properties": {
                    "allowSubdomains": {
                      "type": "boolean",
                      "description": "This option defines who can create subdomains: true - anyone; false - only the domain owner",
                      "$comment": "Only the domain owner is allowed to create subdomains for non top-level domains",
                      "position": 0
                    }
                  },
                  "position": 6,
                  "description": "Subdomain rules allow domain owners to define rules for subdomains",
                  "additionalProperties": false,
                  "required": [
                    "allowSubdomains"
                  ]
                }
              },
              "required": [
                "$createdAt",
                "$updatedAt",
                "$transferredAt",
                "label",
                "normalizedLabel",
                "normalizedParentDomainName",
                "preorderSalt",
                "records",
                "subdomainRules"
              ],
              "transient": [
                "preorderSalt"
              ],
              "additionalProperties": false,
              "$comment": "In order to register a domain you need to create a preorder. The preorder step is needed to prevent man-in-the-middle attacks. normalizedLabel + '.' + normalizedParentDomain must not be longer than 253 chars length as defined by RFC 1035. Domain documents are immutable: modification and deletion are restricted"
            },
            "preorder": {
              "documentsMutable": false,
              "canBeDeleted": true,
              "type": "object",
              "indices": [
                {
                  "name": "saltedHash",
                  "properties": [
                    {
                      "saltedDomainHash": "asc"
                    }
                  ],
                  "unique": true
                }
              ],
              "properties": {
                "saltedDomainHash": {
                  "type": "array",
                  "byteArray": true,
                  "minItems": 32,
                  "maxItems": 32,
                  "position": 0,
                  "description": "Double sha-256 of the concatenation of a 32 byte random salt and a normalized domain name"
                }
              },
              "required": [
                "saltedDomainHash"
              ],
              "additionalProperties": false,
              "$comment": "Preorder documents are immutable: modification and deletion are restricted"
            }
          }
        }
      )
    }
}
