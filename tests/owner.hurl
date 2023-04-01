POST http://localhost:8080/owner
Content-Type: application/json
{
  "payload": {
    "name": "Jack",
    "email": "cyborgninja@sonsofliberty.com",
    "password": "lalilulelo"
  }
}

HTTP 201
[Asserts]
[Captures]
owner_id: jsonpath "$['id']"

GET http://localhost:8080/owner/{{owner_id}}

HTTP 200
[Asserts]


DELETE http://localhost:8080/owner/{{owner_id}}

HTTP 204
[Asserts]
