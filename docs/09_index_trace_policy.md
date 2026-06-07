# 09. Index / Trace / Policy設計

## 9.1 versioned project files

open directory内には以下を置く。

```text
codefire.yaml
codefire.links.yaml
codefire.policy.yaml
```

これらは通常の成果物としてcommit対象である。

## 9.2 codefire.yaml

プロジェクト構成を定義する。

```yaml
version: 1
artifacts:
  requirements:
    - path: docs/spec/**/*.md
      kind: requirement_document
  designs:
    - path: docs/design/**/*.md
      kind: design_document
  adrs:
    - path: adr/**/*.md
      kind: adr_document
  ops:
    - path: docs/ops/**/*.md
      kind: ops_document
  apis:
    - path: docs/api/**/*.json
      kind: openapi_json
  db:
    - path: db/**/*.sql
      kind: sql_schema
  code:
    - path: src/**/*.py
      kind: python_code
  tests:
    - path: tests/**/*.py
      kind: pytest_test
```

## 9.3 Atom ID規約

推奨prefix：

```text
REQ-   requirement
DES-   design
ADR-   architecture decision record
API-   API operation
DB-    database schema element
CODE-  code symbol
TEST-  test case
OPS-   operation/runbook
```

例：

```text
REQ-AUTH-001
DES-AUTH-001
CODE-SessionPolicy
TEST-session-expiration
```

## 9.4 Markdown Atom抽出

Markdownでは、見出し先頭のIDをAtom IDとする。

```markdown
## REQ-AUTH-001: セッション有効期限

ユーザのセッションは最終操作から30分で失効する。
```

同階層以上の次の見出しまでをAtom本文とし、content hashを計算する。
Backtick fenceまたはtilde fenceで囲まれたコードブロック内の見出し風テキストはAtomとして抽出しない。fence開始/終了は最大3つのleading spaceを許容し、通常の見出し抽出と本文scope計算はfence外の見出しだけを対象にする。

## 9.5 OpenAPI Atom抽出

OpenAPI JSON/YAMLでは、`paths` 配下のoperationをAtomとして抽出する。

```json
{
  "paths": {
    "/sessions": {
      "post": {
        "operationId": "createSession",
        "x-codefire-atom-id": "API-AUTH-CREATE-SESSION"
      }
    }
  }
}
```

```yaml
paths:
  /sessions:
    post:
      operationId: createSession
      x-codefire-atom-id: API-AUTH-CREATE-SESSION
```

`x-codefire-atom-id` がある場合はそれをAtom IDにする。ない場合は `operationId` から `API-<operationId>` を生成する。

## 9.6 DB schema Atom抽出

SQL schemaでは、`CREATE TABLE` 文をtable単位とcolumn単位のAtomとして抽出する。`CREATE INDEX` / `CREATE UNIQUE INDEX`、`CREATE VIEW` / `CREATE MATERIALIZED VIEW`、`CREATE TRIGGER`、`CREATE FUNCTION`、`CREATE PROCEDURE`、`CREATE SEQUENCE`、`CREATE TYPE` 文もそれぞれschema object単位のAtomとして抽出する。

```sql
CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions (user_id);

CREATE VIEW active_sessions AS
  SELECT id, user_id FROM sessions WHERE expires_at > CURRENT_TIMESTAMP;

CREATE TRIGGER sessions_touch AFTER UPDATE ON sessions BEGIN
  SELECT NEW.id;
END;

CREATE FUNCTION calculate_session_count() RETURNS INTEGER AS $$
BEGIN
  RETURN 1;
END;
$$ LANGUAGE plpgsql;

CREATE PROCEDURE prune_sessions() AS $$
BEGIN
  DELETE FROM sessions WHERE expires_at < CURRENT_TIMESTAMP;
END;
$$ LANGUAGE SQL;

CREATE SEQUENCE IF NOT EXISTS session_events_seq START WITH 1;

CREATE MATERIALIZED VIEW session_daily_counts AS
  SELECT user_id, count(*) AS total FROM sessions GROUP BY user_id;

CREATE TYPE session_status AS ENUM ('active', 'expired');
```

table名から `DB-<table>` を生成し、次のセミコロンまでをAtom本文としてcontent hashを計算する。
column定義から `DB-<table>.<column>` を生成し、column定義行をAtom本文としてcontent hashを計算する。`CONSTRAINT` / `PRIMARY KEY` / `FOREIGN KEY` / `UNIQUE` / `CHECK` などのtable constraint行はcolumn Atomとして扱わない。
index名から `DB-<index>` を生成し、次のセミコロンまでをAtom本文としてcontent hashを計算する。
view名から `DB-<view>`、trigger名から `DB-<trigger>` を生成し、対象DDL文をAtom本文としてcontent hashを計算する。
function/procedure名から `DB-<routine>` を生成し、対象DDL文をAtom本文としてcontent hashを計算する。PostgreSQL形式のdollar-quoted bodyでは、body内部のセミコロンを文終端として扱わない。
sequence/materialized view/type名からも `DB-<name>` を生成し、対象DDL文をAtom本文としてcontent hashを計算する。

## 9.7 Code Atom抽出

Python / JavaScript / TypeScript / Go / Java / C# / Rust / Kotlin / PHP / Ruby / Swift / C/C++では、明示IDコメントを推奨する。

Atom ID validationはMarkdown headingと明示 `cf-atom:` commentで共通である。許可される形式は、`REQ-...` / `DES-...` / `TEST-...` / `CODE-...` / `ADR-...` / `OPS-...` / `API-...` / `DB-...` のような大文字prefix + `-` + tail、またはPython reference由来の `CODE:<path>::<symbol-kind>:<symbol>` 形式である。tailにはASCII英数字、`_`、`.`、`-` を使える。`CODE:` path形式ではさらに `/` と `:` を使える。無効な明示 `cf-atom:` IDはconfig diagnosticとして報告する。

明示 `cf-atom:` commentのcontent hashは、marker行だけではなく、そのmarker行から次の明示 `cf-atom:` marker直前までのblockを対象にする。1 file内に複数の明示Atomがある場合、各Atomの本文範囲は次のmarkerで区切られる。これにより、同じmarker IDのまま関数・class・test本文だけが変わった場合もAtom変更として検出される。

```python
# cf-atom: CODE-SessionPolicy
class SessionPolicy:
    def expires_after_minutes(self) -> int:
        return 30
```

```ts
// cf-atom: CODE-SessionViewModel
export class SessionViewModel {
  status(): string {
    return "active";
  }
}
```

```go
// cf-atom: CODE-GoSessionStore
type Store struct {
  sessions map[string]string
}

func (s *Store) Find(id string) string {
  return s.sessions[id]
}
```

```java
// cf-atom: CODE-JavaSessionStore
public class SessionStore {
  public String find(String id) {
    return sessions.get(id);
  }
}
```

```csharp
// cf-atom: CODE-CSharpSessionStore
public class SessionStore
{
    public string? Find(string id)
    {
        return sessions.GetValueOrDefault(id);
    }
}
```

```rust
// cf-atom: CODE-RustSessionStore
pub struct SessionStore {
    sessions: HashMap<String, String>,
}

impl SessionStore {
    pub fn find(&self, id: &str) -> Option<&String> {
        self.sessions.get(id)
    }
}
```

```kotlin
// cf-atom: CODE-KotlinSessionStore
class SessionStore(private val sessions: Map<String, String>) {
    fun find(id: String): String? {
        return sessions[id]
    }
}
```

```php
// cf-atom: CODE-PhpSessionStore
class SessionStore {
    public function find(string $id): ?string {
        return $this->sessions[$id] ?? null;
    }
}
```

```ruby
# cf-atom: CODE-RubySessionStore
class SessionStore
  def find(id)
    @sessions[id]
  end
end
```

```swift
// cf-atom: CODE-SwiftSessionStore
struct SessionStore {
    func find(id: String) -> String? {
        return sessions[id]
    }
}
```

```cpp
// cf-atom: CODE-CppSessionStore
class SessionStore {
public:
    std::string find(const std::string& id) const {
        return sessions.at(id);
    }
};
```

明示IDがない場合、Python class/function、JavaScript/TypeScript class/function/arrow function、Go type/function/method、Java type/method、C# type/method、Rust type/function/method、Kotlin type/function/method、PHP type/function/method、Ruby type/function/method、Swift type/function/method、C/C++ type/function/method から派生IDを生成する。

```text
CODE:src/auth.py::class:SessionPolicy
CODE:src/session.ts::function:formatSessionStatus
CODE:src/SessionStore.cs::method:SessionStore.Find
CODE:src/session.rs::method:SessionStore.find
CODE:src/SessionStore.kt::method:SessionStore.find
CODE:src/SessionStore.php::method:SessionStore.find
CODE:src/session_store.rb::method:SessionStore.find
CODE:src/SessionStore.swift::method:SessionStore.find
CODE:src/session_store.cpp::method:SessionStore.find
```

## 9.8 pytest Atom抽出

```python
# cf-atom: TEST-session-expiration
def test_session_expiration():
    assert SessionPolicy().expires_after_minutes() == 30
```

## 9.9 Trace Graph

Trace Linkは `codefire.links.yaml` に定義する。

```yaml
version: 1
links:
  - from: REQ-AUTH-001
    to: DES-AUTH-001
    type: refined_by
  - from: DES-AUTH-001
    to: CODE-SessionPolicy
    type: implemented_by
  - from: REQ-AUTH-001
    to: TEST-session-expiration
    type: verified_by
```

`codefire.links.yaml` はCodeFire schema向けの限定YAML subsetとして読む。`links` 配下は `from` / `to` / `type` のkey-value itemだけをTrace Linkとして受け付ける。`links` section後の別root sectionはTrace Graph入力外metadataとして読み飛ばすが、`links` sectionが無いまま別root sectionだけがある構造、unsupported link field、root外のnested entry、tab indentation、対応外list形式は黙って無視せず、line number付きconfig errorにする。quoted scalarとquoted string内ではないinline commentは処理できる。

新規Trace Linkの内部IDは `LINK-sha256-<32hex>` 形式で、`from` / `to` / `type` のcanonical JSON digestから生成する。旧形式 `LINK-{from}-{type}-{to}` は新規生成には使わないが、既存resolutionのstale判定では旧ID/hashも互換照合する。

Trace Graphを以下の有向グラフとして扱う。

\[
G = (V, E)
\]

変数の定義：

- \( G \)：Trace Graph
- \( V \)：Artifact Atom集合
- \( E \)：Trace Link集合

## 9.10 required link policy

policyでは、Atom種別ごとに必須リンクを定義する。

```yaml
required_links:
  requirement:
    - type: refined_by
      target_kind: design
      min: 1
    - type: verified_by
      target_kind: test
      min: 1
  design:
    - type: implemented_by
      target_kind: code
      min: 1
```

`codefire.policy.yaml` もCodeFire schema向けの限定YAML subsetとして読む。対応root sectionは `version`、`required_links`、`commit_policy`、`extinguish_policy`、`verification` で、各section内のunsupported fieldやunsupported indentationはline number付きconfig errorにする。

必須linkが欠落している場合、commit不可である。

## 9.11 propagation policy

MVPでは単純に、変更Atomに直接接続されたAtomへrequired fireを立てる。

将来的には、変更種別ごとのpolicyを導入する。

```yaml
propagation:
  code_behavior_change:
    implemented_by:
      inverse: required
    verified_by:
      related: required
  requirement_change:
    refined_by:
      forward: required
    verified_by:
      forward: required
```
