# Process-Lib API Documentation

## ast-parser.rs

## sqlite.rs

### Struct `SqliteRequest`

**Fields:**

- `package_id`: `PackageId`
- `db`: `String`
- `action`: `SqliteAction`

### Enum `SqliteAction`

**Variants:**

- `Open`
- `RemoveDb`
- `Write`
- `Query`
- `BeginTx`
- `Commit`

### Enum `SqliteResponse`

**Variants:**

- `Ok`
- `Read`
- `BeginTx`
- `Err`

### Enum `SqlValue`

**Variants:**

- `Integer`
- `Real`
- `Text`
- `Blob`
- `Boolean`
- `Null`

### Enum `SqliteError`

**Variants:**

- `NoDb`
- `NoTx`
- `NoWriteCap`
- `NoReadCap`
- `MismatchingPackageId`
- `AddCapFailed`
- `NotAWriteKeyword`
- `NotAReadKeyword`
- `InvalidParameters`
- `MalformedRequest`
- `RusqliteError`
- `IOError`

### Struct `SqliteCapabilityParams`

**Fields:**

- `kind`: `SqliteCapabilityKind`
- `db_key`: `(PackageId , String)`

### Enum `SqliteCapabilityKind`

**Variants:**

- `Read`
- `Write`

### Struct `Sqlite`

**Fields:**

- `package_id`: `PackageId`
- `db`: `String`
- `timeout`: `u64`

### Impl for `Sqlite`

#### `Sqlite::read` method

```rust
fn read (& self , query : String , params : Vec < serde_json :: Value > ,) -> anyhow :: Result < Vec < HashMap < String , serde_json :: Value > > >
```

#### `Sqlite::write` method

```rust
fn write (& self , statement : String , params : Vec < serde_json :: Value > , tx_id : Option < u64 > ,) -> anyhow :: Result < () >
```

#### `Sqlite::begin_tx` method

```rust
fn begin_tx (& self) -> anyhow :: Result < u64 >
```

#### `Sqlite::commit_tx` method

```rust
fn commit_tx (& self , tx_id : u64) -> anyhow :: Result < () >
```

#### `open` function

```rust
fn open (package_id : PackageId , db : & str , timeout : Option < u64 >) -> anyhow :: Result < Sqlite >
```

#### `remove_db` function

```rust
fn remove_db (package_id : PackageId , db : & str , timeout : Option < u64 >) -> anyhow :: Result < () >
```

## kv.rs

### Struct `KvRequest`

**Fields:**

- `package_id`: `PackageId`
- `db`: `String`
- `action`: `KvAction`

### Enum `KvAction`

**Variants:**

- `Open`
- `RemoveDb`
- `Set`
- `Delete`
- `Get`
- `BeginTx`
- `Commit`

### Enum `KvResponse`

**Variants:**

- `Ok`
- `BeginTx`
- `Get`
- `Err`

### Enum `KvError`

**Variants:**

- `NoDb`
- `KeyNotFound`
- `NoTx`
- `NoWriteCap`
- `NoReadCap`
- `MismatchingPackageId`
- `AddCapFailed`
- `MalformedRequest`
- `RocksDBError`
- `IOError`

### Struct `KvCapabilityParams`

**Fields:**

- `kind`: `KvCapabilityKind`
- `db_key`: `(PackageId , String)`

### Enum `KvCapabilityKind`

**Variants:**

- `Read`
- `Write`

### Struct `Kv`

**Fields:**

- `package_id`: `PackageId`
- `db`: `String`
- `timeout`: `u64`

### Impl for `Kv < K , V >`

#### `Kv < K , V >::get` method

```rust
fn get (& self , key : & K) -> anyhow :: Result < V >
```

#### `Kv < K , V >::get_as` method

```rust
fn get_as < T > (& self , key : & K) -> anyhow :: Result < T > where T : DeserializeOwned ,
```

#### `Kv < K , V >::set` method

```rust
fn set (& self , key : & K , value : & V , tx_id : Option < u64 >) -> anyhow :: Result < () >
```

#### `Kv < K , V >::set_as` method

```rust
fn set_as < T > (& self , key : & K , value : & T , tx_id : Option < u64 >) -> anyhow :: Result < () > where T : Serialize ,
```

#### `Kv < K , V >::delete` method

```rust
fn delete (& self , key : & K , tx_id : Option < u64 >) -> anyhow :: Result < () >
```

#### `Kv < K , V >::delete_as` method

```rust
fn delete_as < T > (& self , key : & T , tx_id : Option < u64 >) -> anyhow :: Result < () > where T : Serialize ,
```

#### `Kv < K , V >::begin_tx` method

```rust
fn begin_tx (& self) -> anyhow :: Result < u64 >
```

#### `Kv < K , V >::commit_tx` method

```rust
fn commit_tx (& self , tx_id : u64) -> anyhow :: Result < () >
```

### Impl for `Kv < Vec < u8 > , Vec < u8 > >`

#### `Kv < Vec < u8 > , Vec < u8 > >::get_raw` method

```rust
fn get_raw (& self , key : & [u8]) -> anyhow :: Result < Vec < u8 > >
```

#### `Kv < Vec < u8 > , Vec < u8 > >::set_raw` method

```rust
fn set_raw (& self , key : & [u8] , value : & [u8] , tx_id : Option < u64 >) -> anyhow :: Result < () >
```

#### `Kv < Vec < u8 > , Vec < u8 > >::delete_raw` method

```rust
fn delete_raw (& self , key : & [u8] , tx_id : Option < u64 >) -> anyhow :: Result < () >
```

#### `open_raw` function

```rust
fn open_raw (package_id : PackageId , db : & str , timeout : Option < u64 > ,) -> anyhow :: Result < Kv < Vec < u8 > , Vec < u8 > > >
```

#### `open` function

```rust
fn open < K , V > (package_id : PackageId , db : & str , timeout : Option < u64 >) -> anyhow :: Result < Kv < K , V > > where K : Serialize + DeserializeOwned , V : Serialize + DeserializeOwned ,
```

#### `remove_db` function

```rust
fn remove_db (package_id : PackageId , db : & str , timeout : Option < u64 >) -> anyhow :: Result < () >
```

## mod.rs

### Struct `VfsRequest`

**Fields:**

- `path`: `String`
- `action`: `VfsAction`

### Enum `VfsAction`

**Variants:**

- `CreateDrive`
- `CreateDir`
- `CreateDirAll`
- `CreateFile`
- `OpenFile`
- `CloseFile`
- `Write`
- `WriteAll`
- `Append`
- `SyncAll`
- `Read`
- `ReadDir`
- `ReadToEnd`
- `ReadExact`
- `ReadToString`
- `Seek`
- `RemoveFile`
- `RemoveDir`
- `RemoveDirAll`
- `Rename`
- `Metadata`
- `AddZip`
- `CopyFile`
- `Len`
- `SetLen`
- `Hash`

### Enum `SeekFrom`

**Variants:**

- `Start`
- `End`
- `Current`

### Enum `FileType`

**Variants:**

- `File`
- `Directory`
- `Symlink`
- `Other`

### Struct `FileMetadata`

**Fields:**

- `file_type`: `FileType`
- `len`: `u64`

### Struct `DirEntry`

**Fields:**

- `path`: `String`
- `file_type`: `FileType`

### Enum `VfsResponse`

**Variants:**

- `Ok`
- `Err`
- `Read`
- `SeekFrom`
- `ReadDir`
- `ReadToString`
- `Metadata`
- `Len`
- `Hash`

### Enum `VfsError`

**Variants:**

- `NoWriteCap`
- `NoReadCap`
- `AddCapFailed`
- `MalformedRequest`
- `NoBlob`
- `ParseError`
- `IOError`
- `UnzipError`
- `SendError`

#### `vfs_request` function

```rust
fn vfs_request < T > (path : T , action : VfsAction) -> Request where T : Into < String > ,
```

#### `metadata` function

```rust
fn metadata (path : & str , timeout : Option < u64 >) -> Result < FileMetadata , VfsError >
```

#### `remove_path` function

```rust
fn remove_path (path : & str , timeout : Option < u64 >) -> Result < () , VfsError >
```

#### `parse_response` function

```rust
fn parse_response (body : & [u8]) -> Result < VfsResponse , VfsError >
```

## directory.rs

### Struct `Directory`

**Fields:**

- `path`: `String`
- `timeout`: `u64`

### Impl for `Directory`

#### `Directory::read` method

```rust
fn read (& self) -> Result < Vec < DirEntry > , VfsError >
```

#### `open_dir` function

```rust
fn open_dir (path : & str , create : bool , timeout : Option < u64 >) -> Result < Directory , VfsError >
```

#### `remove_dir` function

```rust
fn remove_dir (path : & str , timeout : Option < u64 >) -> Result < () , VfsError >
```

## file.rs

### Struct `File`

**Fields:**

- `path`: `String`
- `timeout`: `u64`

### Impl for `File`

#### `File::new` method

```rust
fn new < T : Into < String > > (path : T , timeout : u64) -> Self
```

#### `File::read` method

```rust
fn read (& self) -> Result < Vec < u8 > , VfsError >
```

#### `File::read_into` method

```rust
fn read_into (& self , buffer : & mut [u8]) -> Result < usize , VfsError >
```

#### `File::read_at` method

```rust
fn read_at (& self , buffer : & mut [u8]) -> Result < usize , VfsError >
```

#### `File::read_to_end` method

```rust
fn read_to_end (& self) -> Result < Vec < u8 > , VfsError >
```

#### `File::read_to_string` method

```rust
fn read_to_string (& self) -> Result < String , VfsError >
```

#### `File::write` method

```rust
fn write (& self , buffer : & [u8]) -> Result < () , VfsError >
```

#### `File::write_all` method

```rust
fn write_all (& mut self , buffer : & [u8]) -> Result < () , VfsError >
```

#### `File::append` method

```rust
fn append (& mut self , buffer : & [u8]) -> Result < () , VfsError >
```

#### `File::seek` method

```rust
fn seek (& mut self , pos : SeekFrom) -> Result < u64 , VfsError >
```

#### `File::copy` method

```rust
fn copy (& mut self , path : & str) -> Result < File , VfsError >
```

#### `File::set_len` method

```rust
fn set_len (& mut self , size : u64) -> Result < () , VfsError >
```

#### `File::metadata` method

```rust
fn metadata (& self) -> Result < FileMetadata , VfsError >
```

#### `File::sync_all` method

```rust
fn sync_all (& self) -> Result < () , VfsError >
```

### Impl for `File`

#### `File::drop` method

```rust
fn drop (& mut self)
```

#### `create_drive` function

```rust
fn create_drive (package_id : PackageId , drive : & str , timeout : Option < u64 > ,) -> Result < String , VfsError >
```

#### `open_file` function

```rust
fn open_file (path : & str , create : bool , timeout : Option < u64 >) -> Result < File , VfsError >
```

#### `create_file` function

```rust
fn create_file (path : & str , timeout : Option < u64 >) -> Result < File , VfsError >
```

#### `remove_file` function

```rust
fn remove_file (path : & str , timeout : Option < u64 >) -> Result < () , VfsError >
```

## message.rs

### Enum `Message`

**Variants:**

- `Request`
- `Response`

### Enum `BuildError`

**Variants:**

- `NoBody`
- `NoTarget`

### Impl for `Message`

#### `Message::source` method

```rust
fn source (& self) -> & Address
```

#### `Message::body` method

```rust
fn body (& self) -> & [u8]
```

#### `Message::metadata` method

```rust
fn metadata (& self) -> Option < & str >
```

#### `Message::context` method

```rust
fn context (& self) -> Option < & [u8] >
```

#### `Message::blob` method

```rust
fn blob (& self) -> Option < LazyLoadBlob >
```

#### `Message::capabilities` method

```rust
fn capabilities (& self) -> & Vec < Capability >
```

#### `Message::is_request` method

```rust
fn is_request (& self) -> bool
```

#### `Message::is_local` method

```rust
fn is_local (& self) -> bool
```

#### `Message::is_process` method

```rust
fn is_process < T > (& self , process : T) -> bool where ProcessId : PartialEq < T > ,
```

#### `_wit_message_to_message` function

```rust
fn _wit_message_to_message (source : Address , message : crate :: hyperware :: process :: standard :: Message ,) -> Message
```

## response.rs

### Struct `Response`

**Fields:**


### Impl for `Response`

#### `Response::new` method

```rust
fn new () -> Self
```

#### `Response::inherit` method

```rust
fn inherit (mut self , inherit : bool) -> Self
```

#### `Response::body` method

```rust
fn body < T > (mut self , body : T) -> Self where T : Into < Vec < u8 > > ,
```

#### `Response::try_body` method

```rust
fn try_body < T , E > (mut self , body : T) -> Result < Self , E > where T : TryInto < Vec < u8 > , Error = E > , E : std :: error :: Error ,
```

#### `Response::metadata` method

```rust
fn metadata (mut self , metadata : & str) -> Self
```

#### `Response::blob` method

```rust
fn blob (mut self , blob : LazyLoadBlob) -> Self
```

#### `Response::blob_mime` method

```rust
fn blob_mime (mut self , mime : & str) -> Self
```

#### `Response::blob_bytes` method

```rust
fn blob_bytes < T > (mut self , bytes : T) -> Self where T : Into < Vec < u8 > > ,
```

#### `Response::try_blob_bytes` method

```rust
fn try_blob_bytes < T , E > (mut self , bytes : T) -> Result < Self , E > where T : TryInto < Vec < u8 > , Error = E > , E : std :: error :: Error ,
```

#### `Response::capabilities` method

```rust
fn capabilities (mut self , capabilities : Vec < Capability >) -> Self
```

#### `Response::attach_all` method

```rust
fn attach_all (mut self , target : & Address) -> Self
```

#### `Response::send` method

```rust
fn send (self) -> Result < () , BuildError >
```

### Impl for `Response`

#### `Response::default` method

```rust
fn default () -> Self
```

## request.rs

### Struct `Request`

**Fields:**

- `target`: `Option < Address >`
- `inherit`: `bool`
- `timeout`: `Option < u64 >`
- `body`: `Option < Vec < u8 > >`
- `metadata`: `Option < String >`
- `blob`: `Option < LazyLoadBlob >`
- `context`: `Option < Vec < u8 > >`
- `capabilities`: `Vec < Capability >`

### Impl for `Request`

#### `Request::new` method

```rust
fn new () -> Self
```

#### `Request::to` method

```rust
fn to < T > (target : T) -> Self where T : Into < Address > ,
```

#### `Request::target` method

```rust
fn target < T > (mut self , target : T) -> Self where T : Into < Address > ,
```

#### `Request::inherit` method

```rust
fn inherit (mut self , inherit : bool) -> Self
```

#### `Request::expects_response` method

```rust
fn expects_response (mut self , timeout : u64) -> Self
```

#### `Request::body` method

```rust
fn body < T > (mut self , body : T) -> Self where T : Into < Vec < u8 > > ,
```

#### `Request::try_body` method

```rust
fn try_body < T , E > (mut self , body : T) -> Result < Self , E > where T : TryInto < Vec < u8 > , Error = E > , E : std :: error :: Error ,
```

#### `Request::metadata` method

```rust
fn metadata (mut self , metadata : & str) -> Self
```

#### `Request::blob` method

```rust
fn blob (mut self , blob : LazyLoadBlob) -> Self
```

#### `Request::blob_mime` method

```rust
fn blob_mime (mut self , mime : & str) -> Self
```

#### `Request::blob_bytes` method

```rust
fn blob_bytes < T > (mut self , bytes : T) -> Self where T : Into < Vec < u8 > > ,
```

#### `Request::try_blob_bytes` method

```rust
fn try_blob_bytes < T , E > (mut self , bytes : T) -> Result < Self , E > where T : TryInto < Vec < u8 > , Error = E > , E : std :: error :: Error ,
```

#### `Request::context` method

```rust
fn context < T > (mut self , context : T) -> Self where T : Into < Vec < u8 > > ,
```

#### `Request::try_context` method

```rust
fn try_context < T , E > (mut self , context : T) -> Result < Self , E > where T : TryInto < Vec < u8 > , Error = E > , E : std :: error :: Error ,
```

#### `Request::capabilities` method

```rust
fn capabilities (mut self , capabilities : Vec < Capability >) -> Self
```

#### `Request::attach_messaging` method

```rust
fn attach_messaging (mut self) -> Self
```

#### `Request::try_attach_all` method

```rust
fn try_attach_all (self) -> Result < Self , BuildError >
```

#### `Request::attach_all` method

```rust
fn attach_all (mut self , target : & Address) -> Self
```

#### `Request::send` method

```rust
fn send (self) -> Result < () , BuildError >
```

#### `Request::send_and_await_response` method

```rust
fn send_and_await_response (self , timeout : u64 ,) -> Result < Result < Message , SendError > , BuildError >
```

### Impl for `Request`

#### `Request::default` method

```rust
fn default () -> Self
```

## send_error.rs

### Struct `SendError`

**Fields:**

- `kind`: `SendErrorKind`
- `target`: `Address`
- `message`: `Message`
- `lazy_load_blob`: `Option < LazyLoadBlob >`
- `context`: `Option < Vec < u8 > >`

### Impl for `SendError`

#### `SendError::kind` method

```rust
fn kind (& self) -> & SendErrorKind
```

#### `SendError::target` method

```rust
fn target (& self) -> & Address
```

#### `SendError::message` method

```rust
fn message (& self) -> & Message
```

#### `SendError::blob` method

```rust
fn blob (& self) -> Option < & LazyLoadBlob >
```

#### `SendError::context` method

```rust
fn context (& self) -> Option < & [u8] >
```

### Impl for `SendError`

#### `SendError::fmt` method

```rust
fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result
```

### Impl for `SendError`

#### `SendError::description` method

```rust
fn description (& self) -> & str
```

### Enum `SendErrorKind`

**Variants:**

- `Offline`
- `Timeout`

### Impl for `SendErrorKind`

#### `SendErrorKind::is_offline` method

```rust
fn is_offline (& self) -> bool
```

#### `SendErrorKind::is_timeout` method

```rust
fn is_timeout (& self) -> bool
```

#### `_wit_send_error_to_send_error` function

```rust
fn _wit_send_error_to_send_error (send_err : crate :: hyperware :: process :: standard :: SendError , context : Option < Vec < u8 > > ,) -> SendError
```

## on_exit.rs

### Enum `OnExit`

**Variants:**

- `None`
- `Restart`
- `Requests`

### Impl for `OnExit`

#### `OnExit::get` method

```rust
fn get () -> Self
```

#### `OnExit::is_none` method

```rust
fn is_none (& self) -> bool
```

#### `OnExit::is_restart` method

```rust
fn is_restart (& self) -> bool
```

#### `OnExit::is_requests` method

```rust
fn is_requests (& self) -> bool
```

#### `OnExit::get_requests` method

```rust
fn get_requests (& self) -> Option < & [Request] >
```

#### `OnExit::add_request` method

```rust
fn add_request (& mut self , new : Request)
```

#### `OnExit::set` method

```rust
fn set (self) -> Result < () , BuildError >
```

#### `OnExit::_to_standard` method

```rust
fn _to_standard (self) -> Result < crate :: hyperware :: process :: standard :: OnExit , BuildError >
```

## address.rs

### Impl for `Address`

#### `Address::new` method

```rust
fn new < T , U > (node : T , process : U) -> Address where T : Into < String > , U : Into < ProcessId > ,
```

#### `Address::node` method

```rust
fn node (& self) -> & str
```

#### `Address::process` method

```rust
fn process (& self) -> & str
```

#### `Address::package` method

```rust
fn package (& self) -> & str
```

#### `Address::publisher` method

```rust
fn publisher (& self) -> & str
```

#### `Address::package_id` method

```rust
fn package_id (& self) -> crate :: PackageId
```

#### `Address::send_request` method

```rust
fn send_request (& self) -> Request
```

### Impl for `Address`

#### `Address::from_str` method

```rust
fn from_str (input : & str) -> Result < Self , AddressParseError >
```

### Impl for `Address`

#### `Address::serialize` method

```rust
fn serialize < S > (& self , serializer : S) -> Result < S :: Ok , S :: Error > where S : serde :: ser :: Serializer ,
```

### Impl for `Address`

#### `Address::deserialize` method

```rust
fn deserialize < D > (deserializer : D) -> Result < Address , D :: Error > where D : serde :: de :: Deserializer < 'a > ,
```

### Impl for `Address`

#### `Address::hash` method

```rust
fn hash < H : Hasher > (& self , state : & mut H)
```

### Impl for `Address`

### Impl for `Address`

#### `Address::eq` method

```rust
fn eq (& self , other : & Self) -> bool
```

### Impl for `Address`

#### `Address::from` method

```rust
fn from (input : & Address) -> Self
```

### Impl for `Address`

#### `Address::from` method

```rust
fn from (input : (T , U , V , W)) -> Self
```

### Impl for `Address`

#### `Address::from` method

```rust
fn from (input : (T , U)) -> Self
```

### Impl for `Address`

#### `Address::fmt` method

```rust
fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result
```

### Enum `AddressParseError`

**Variants:**

- `TooManyAts`
- `TooManyColons`
- `MissingNodeId`
- `MissingField`

### Impl for `AddressParseError`

#### `AddressParseError::fmt` method

```rust
fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result
```

### Impl for `AddressParseError`

#### `AddressParseError::description` method

```rust
fn description (& self) -> & str
```

## process_id.rs

### Impl for `ProcessId`

#### `ProcessId::new` method

```rust
fn new (process_name : Option < & str > , package_name : & str , publisher_node : & str) -> Self
```

#### `ProcessId::process` method

```rust
fn process (& self) -> & str
```

#### `ProcessId::package` method

```rust
fn package (& self) -> & str
```

#### `ProcessId::publisher` method

```rust
fn publisher (& self) -> & str
```

### Impl for `ProcessId`

#### `ProcessId::from_str` method

```rust
fn from_str (input : & str) -> Result < Self , ProcessIdParseError >
```

### Impl for `ProcessId`

#### `ProcessId::serialize` method

```rust
fn serialize < S > (& self , serializer : S) -> Result < S :: Ok , S :: Error > where S : serde :: ser :: Serializer ,
```

### Impl for `ProcessId`

#### `ProcessId::deserialize` method

```rust
fn deserialize < D > (deserializer : D) -> Result < ProcessId , D :: Error > where D : serde :: de :: Deserializer < 'a > ,
```

### Impl for `ProcessId`

#### `ProcessId::hash` method

```rust
fn hash < H : Hasher > (& self , state : & mut H)
```

### Impl for `ProcessId`

### Impl for `ProcessId`

#### `ProcessId::from` method

```rust
fn from (input : (& str , & str , & str)) -> Self
```

### Impl for `ProcessId`

#### `ProcessId::fmt` method

```rust
fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result
```

### Impl for `ProcessId`

#### `ProcessId::eq` method

```rust
fn eq (& self , other : & Self) -> bool
```

### Impl for `ProcessId`

#### `ProcessId::eq` method

```rust
fn eq (& self , other : & & str) -> bool
```

### Impl for `& str`

#### `& str::eq` method

```rust
fn eq (& self , other : & ProcessId) -> bool
```

### Enum `ProcessIdParseError`

**Variants:**

- `TooManyColons`
- `MissingField`
- `InvalidCharacter`

### Impl for `ProcessIdParseError`

#### `ProcessIdParseError::fmt` method

```rust
fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result
```

### Impl for `ProcessIdParseError`

#### `ProcessIdParseError::description` method

```rust
fn description (& self) -> & str
```

## mod.rs

## lazy_load_blob.rs

### Impl for `LazyLoadBlob`

#### `LazyLoadBlob::new` method

```rust
fn new < T , U > (mime : Option < T > , bytes : U) -> LazyLoadBlob where T : Into < String > , U : Into < Vec < u8 > > ,
```

#### `LazyLoadBlob::mime` method

```rust
fn mime (& self) -> Option < & str >
```

#### `LazyLoadBlob::bytes` method

```rust
fn bytes (& self) -> & [u8]
```

### Impl for `LazyLoadBlob`

#### `LazyLoadBlob::default` method

```rust
fn default () -> Self
```

### Impl for `LazyLoadBlob`

#### `LazyLoadBlob::eq` method

```rust
fn eq (& self , other : & Self) -> bool
```

## package_id.rs

### Impl for `PackageId`

#### `PackageId::new` method

```rust
fn new (package_name : & str , publisher_node : & str) -> Self
```

#### `PackageId::package` method

```rust
fn package (& self) -> & str
```

#### `PackageId::publisher` method

```rust
fn publisher (& self) -> & str
```

### Impl for `PackageId`

#### `PackageId::serialize` method

```rust
fn serialize < S > (& self , serializer : S) -> Result < S :: Ok , S :: Error > where S : serde :: ser :: Serializer ,
```

### Impl for `PackageId`

#### `PackageId::deserialize` method

```rust
fn deserialize < D > (deserializer : D) -> Result < Self , D :: Error > where D : serde :: de :: Deserializer < 'de > ,
```

### Impl for `PackageId`

#### `PackageId::from_str` method

```rust
fn from_str (input : & str) -> Result < Self , Self :: Err >
```

### Impl for `PackageId`

#### `PackageId::hash` method

```rust
fn hash < H : std :: hash :: Hasher > (& self , state : & mut H)
```

### Impl for `PackageId`

#### `PackageId::fmt` method

```rust
fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result
```

### Impl for `PackageId`

#### `PackageId::from` method

```rust
fn from (input : (& str , & str)) -> Self
```

### Impl for `PackageId`

### Impl for `PackageId`

#### `PackageId::eq` method

```rust
fn eq (& self , other : & Self) -> bool
```

## capability.rs

### Impl for `Capability`

#### `Capability::new` method

```rust
fn new < T , U > (address : T , params : U) -> Capability where T : Into < Address > , U : Into < String > ,
```

#### `Capability::issuer` method

```rust
fn issuer (& self) -> & Address
```

#### `Capability::params` method

```rust
fn params (& self) -> & str
```

#### `Capability::params_json` method

```rust
fn params_json (& self) -> Result < serde_json :: Value , serde_json :: Error >
```

#### `Capability::set_params_json` method

```rust
fn set_params_json (& mut self , value : serde_json :: Value) -> Result < () , serde_json :: Error >
```

### Impl for `Capability`

#### `Capability::serialize` method

```rust
fn serialize < S > (& self , serializer : S) -> Result < S :: Ok , S :: Error > where S : serde :: ser :: Serializer ,
```

### Impl for `Capability`

#### `Capability::deserialize` method

```rust
fn deserialize < D > (deserializer : D) -> Result < Capability , D :: Error > where D : serde :: de :: Deserializer < 'a > ,
```

### Impl for `Capability`

#### `Capability::hash` method

```rust
fn hash < H : Hasher > (& self , state : & mut H)
```

### Impl for `Capability`

### Impl for `Capability`

#### `Capability::eq` method

```rust
fn eq (& self , other : & Self) -> bool
```

### Impl for `Capability`

#### `Capability::from` method

```rust
fn from (input : & Capability) -> Self
```

### Impl for `Capability`

#### `Capability::from` method

```rust
fn from (input : (T , U)) -> Self
```

### Impl for `Capability`

#### `Capability::fmt` method

```rust
fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result
```

## timer.rs

### Enum `TimerAction`

**Variants:**

- `Debug`
- `SetTimer`

### Impl for `TimerAction`

#### `TimerAction::into` method

```rust
fn into (self) -> Vec < u8 >
```

#### `set_timer` function

```rust
fn set_timer (duration : u64 , context : Option < Context >)
```

#### `set_and_await_timer` function

```rust
fn set_and_await_timer (duration : u64) -> Result < Message , SendError >
```

## mod.rs

## net.rs

### Struct `Identity`

**Fields:**

- `name`: `NodeId`
- `networking_key`: `String`
- `routing`: `NodeRouting`

### Enum `NodeRouting`

**Variants:**

- `Routers`
- `Direct`

### Impl for `Identity`

#### `Identity::is_direct` method

```rust
fn is_direct (& self) -> bool
```

#### `Identity::get_protocol_port` method

```rust
fn get_protocol_port (& self , protocol : & str) -> Option < u16 >
```

#### `Identity::routers` method

```rust
fn routers (& self) -> Option < & Vec < NodeId > >
```

### Enum `NetAction`

**Variants:**

- `ConnectionRequest`
- `HnsUpdate`
- `HnsBatchUpdate`
- `GetPeers`
- `GetPeer`
- `GetDiagnostics`
- `Sign`
- `Verify`

### Enum `NetResponse`

**Variants:**

- `Accepted`
- `Rejected`
- `Peers`
- `Peer`
- `Diagnostics`
- `Signed`
- `Verified`

### Enum `IndexerRequests`

**Variants:**

- `NamehashToName`

### Struct `NamehashToNameRequest`

**Fields:**

- `hash`: `String`
- `block`: `u64`

### Enum `IndexerResponses`

**Variants:**

- `Name`

### Struct `HnsUpdate`

**Fields:**

- `name`: `String`
- `public_key`: `String`
- `ips`: `Vec < String >`
- `ports`: `BTreeMap < String , u16 >`
- `routers`: `Vec < String >`

### Impl for `HnsUpdate`

#### `HnsUpdate::get_protocol_port` method

```rust
fn get_protocol_port (& self , protocol : & str) -> u16
```

#### `sign` function

```rust
fn sign < T > (message : T) -> Result < Vec < u8 > , SendError > where T : Into < Vec < u8 > > ,
```

#### `verify` function

```rust
fn verify < T , U , V > (from : T , message : U , signature : V) -> Result < bool , SendError > where T : Into < Address > , U : Into < Vec < u8 > > , V : Into < Vec < u8 > > ,
```

#### `get_name` function

```rust
fn get_name < T > (namehash : T , block : Option < u64 > , timeout : Option < u64 >) -> Option < String > where T : Into < String > ,
```

## lib.rs

#### `await_message` function

```rust
fn await_message () -> Result < Message , SendError >
```

#### `await_next_message_body` function

```rust
fn await_next_message_body () -> Result < Vec < u8 > , SendError >
```

#### `spawn` function

```rust
fn spawn (name : Option < & str > , wasm_path : & str , on_exit : OnExit , request_capabilities : Vec < Capability > , grant_capabilities : Vec < (ProcessId , Json) > , public : bool ,) -> Result < ProcessId , SpawnError >
```

#### `make_blob` function

```rust
fn make_blob < T , F , E > (blob : & T , serializer : F) -> Result < LazyLoadBlob , E > where F : Fn (& T) -> Result < Vec < u8 > , E > , E : std :: error :: Error ,
```

#### `get_typed_blob` function

```rust
fn get_typed_blob < T , F , E > (deserializer : F) -> Option < T > where F : Fn (& [u8]) -> Result < T , E > , E : std :: error :: Error ,
```

#### `get_typed_state` function

```rust
fn get_typed_state < T , F , E > (deserializer : F) -> Option < T > where F : Fn (& [u8]) -> Result < T , E > , E : std :: error :: Error ,
```

#### `can_message` function

```rust
fn can_message (address : & Address) -> bool
```

#### `get_capability` function

```rust
fn get_capability (issuer : & Address , params : & str) -> Option < Capability >
```

## logging.rs

### Struct `RemoteLogSettings`

**Fields:**

- `target`: `Address`
- `level`: `Level`

### Struct `RemoteWriter`

**Fields:**

- `target`: `Address`

### Struct `RemoteWriterMaker`

**Fields:**

- `target`: `Address`

### Struct `FileWriter`

**Fields:**

- `file`: `File`
- `max_size`: `u64`

### Struct `FileWriterMaker`

**Fields:**

- `file`: `File`
- `max_size`: `u64`

### Struct `TerminalWriter`

**Fields:**

- `level`: `u8`

### Struct `TerminalWriterMaker`

**Fields:**

- `level`: `u8`

### Impl for `RemoteWriter`

#### `RemoteWriter::write` method

```rust
fn write (& mut self , buf : & [u8]) -> std :: io :: Result < usize >
```

#### `RemoteWriter::flush` method

```rust
fn flush (& mut self) -> std :: io :: Result < () >
```

### Impl for `RemoteWriterMaker`

#### `RemoteWriterMaker::make_writer` method

```rust
fn make_writer (& 'a self) -> Self :: Writer
```

### Impl for `FileWriter`

#### `FileWriter::write` method

```rust
fn write (& mut self , buf : & [u8]) -> std :: io :: Result < usize >
```

#### `FileWriter::flush` method

```rust
fn flush (& mut self) -> std :: io :: Result < () >
```

### Impl for `FileWriterMaker`

#### `FileWriterMaker::make_writer` method

```rust
fn make_writer (& 'a self) -> Self :: Writer
```

### Impl for `TerminalWriter`

#### `TerminalWriter::write` method

```rust
fn write (& mut self , buf : & [u8]) -> std :: io :: Result < usize >
```

#### `TerminalWriter::flush` method

```rust
fn flush (& mut self) -> std :: io :: Result < () >
```

### Impl for `TerminalWriterMaker`

#### `TerminalWriterMaker::make_writer` method

```rust
fn make_writer (& 'a self) -> Self :: Writer
```

#### `init_logging` function

```rust
fn init_logging (file_level : Level , terminal_level : Level , remote : Option < RemoteLogSettings > , terminal_levels_mapping : Option < (u8 , u8 , u8 , u8) > , max_log_file_size : Option < u64 > ,) -> anyhow :: Result < () >
```

## eth.rs

### Enum `SubscriptionKind`

**Variants:**

- `NewHeads`
- `Logs`
- `NewPendingTransactions`
- `Syncing`

### Enum `EthAction`

**Variants:**

- `SubscribeLogs`
- `UnsubscribeLogs`
- `Request`

### Struct `EthSub`

**Fields:**

- `id`: `u64`
- `result`: `serde_json :: Value`

### Struct `EthSubError`

**Fields:**

- `id`: `u64`
- `error`: `String`

### Enum `EthResponse`

**Variants:**

- `Ok`
- `Response`
- `Err`

### Enum `EthError`

**Variants:**

- `RpcError`
- `MalformedRequest`
- `NoRpcForChain`
- `SubscriptionClosed`
- `InvalidMethod`
- `InvalidParams`
- `PermissionDenied`
- `RpcTimeout`
- `RpcMalformedResponse`

### Impl for `EthError`

#### `EthError::fmt` method

```rust
fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result
```

### Impl for `EthError`

### Enum `EthConfigAction`

**Variants:**

- `AddProvider`
- `RemoveProvider`
- `SetPublic`
- `SetPrivate`
- `AllowNode`
- `UnallowNode`
- `DenyNode`
- `UndenyNode`
- `SetProviders`
- `GetProviders`
- `GetAccessSettings`
- `GetState`

### Enum `EthConfigResponse`

**Variants:**

- `Ok`
- `Providers`
- `AccessSettings`
- `PermissionDenied`
- `State`

### Struct `AccessSettings`

**Fields:**

- `public`: `bool`
- `allow`: `HashSet < String >`
- `deny`: `HashSet < String >`

### Struct `ProviderConfig`

**Fields:**

- `chain_id`: `u64`
- `trusted`: `bool`
- `provider`: `NodeOrRpcUrl`

### Enum `Authorization`

**Variants:**

- `Basic`
- `Bearer`
- `Raw`

### Impl for `AlloyAuthorization`

#### `AlloyAuthorization::from` method

```rust
fn from (auth : Authorization) -> AlloyAuthorization
```

### Enum `NodeOrRpcUrl`

**Variants:**

- `Node`
- `RpcUrl`

### Impl for `NodeOrRpcUrl`

#### `NodeOrRpcUrl::eq` method

```rust
fn eq (& self , other : & str) -> bool
```

### Impl for `NodeOrRpcUrl`

#### `NodeOrRpcUrl::deserialize` method

```rust
fn deserialize < D > (deserializer : D) -> Result < Self , D :: Error > where D : serde :: Deserializer < 'de > ,
```

### Struct `Provider`

**Fields:**


### Impl for `Provider`

#### `Provider::new` method

```rust
fn new (chain_id : u64 , request_timeout : u64) -> Self
```

#### `Provider::send_request_and_parse_response` method

```rust
fn send_request_and_parse_response < T : serde :: de :: DeserializeOwned > (& self , action : EthAction ,) -> Result < T , EthError >
```

#### `Provider::get_block_number` method

```rust
fn get_block_number (& self) -> Result < u64 , EthError >
```

#### `Provider::get_balance` method

```rust
fn get_balance (& self , address : Address , tag : Option < BlockId >) -> Result < U256 , EthError >
```

#### `Provider::get_logs` method

```rust
fn get_logs (& self , filter : & Filter) -> Result < Vec < Log > , EthError >
```

#### `Provider::get_gas_price` method

```rust
fn get_gas_price (& self) -> Result < U256 , EthError >
```

#### `Provider::get_transaction_count` method

```rust
fn get_transaction_count (& self , address : Address , tag : Option < BlockId > ,) -> Result < U256 , EthError >
```

#### `Provider::get_block_by_hash` method

```rust
fn get_block_by_hash (& self , hash : BlockHash , full_tx : bool ,) -> Result < Option < Block > , EthError >
```

#### `Provider::get_block_by_number` method

```rust
fn get_block_by_number (& self , number : BlockNumberOrTag , full_tx : bool ,) -> Result < Option < Block > , EthError >
```

#### `Provider::get_storage_at` method

```rust
fn get_storage_at (& self , address : Address , key : U256 , tag : Option < BlockId > ,) -> Result < Bytes , EthError >
```

#### `Provider::get_code_at` method

```rust
fn get_code_at (& self , address : Address , tag : BlockId) -> Result < Bytes , EthError >
```

#### `Provider::get_transaction_by_hash` method

```rust
fn get_transaction_by_hash (& self , hash : TxHash) -> Result < Option < Transaction > , EthError >
```

#### `Provider::get_transaction_receipt` method

```rust
fn get_transaction_receipt (& self , hash : TxHash ,) -> Result < Option < TransactionReceipt > , EthError >
```

#### `Provider::estimate_gas` method

```rust
fn estimate_gas (& self , tx : TransactionRequest , block : Option < BlockId > ,) -> Result < U256 , EthError >
```

#### `Provider::get_accounts` method

```rust
fn get_accounts (& self) -> Result < Vec < Address > , EthError >
```

#### `Provider::get_fee_history` method

```rust
fn get_fee_history (& self , block_count : U256 , last_block : BlockNumberOrTag , reward_percentiles : Vec < f64 > ,) -> Result < FeeHistory , EthError >
```

#### `Provider::call` method

```rust
fn call (& self , tx : TransactionRequest , block : Option < BlockId >) -> Result < Bytes , EthError >
```

#### `Provider::hypermap` method

```rust
fn hypermap (& self) -> crate :: hypermap :: Hypermap
```

#### `Provider::hypermap_with_address` method

```rust
fn hypermap_with_address (self , address : Address) -> crate :: hypermap :: Hypermap
```

#### `Provider::send_raw_transaction` method

```rust
fn send_raw_transaction (& self , tx : Bytes) -> Result < TxHash , EthError >
```

#### `Provider::subscribe` method

```rust
fn subscribe (& self , sub_id : u64 , filter : Filter) -> Result < () , EthError >
```

#### `Provider::subscribe_loop` method

```rust
fn subscribe_loop (& self , sub_id : u64 , filter : Filter , print_verbosity_success : u8 , print_verbosity_error : u8 ,)
```

#### `Provider::unsubscribe` method

```rust
fn unsubscribe (& self , sub_id : u64) -> Result < () , EthError >
```

## hypermap.rs

### Struct `Mint`

**Fields:**

- `name`: `String`
- `parent_path`: `String`

### Struct `Note`

**Fields:**

- `note`: `String`
- `parent_path`: `String`
- `data`: `Bytes`

### Struct `Fact`

**Fields:**

- `fact`: `String`
- `parent_path`: `String`
- `data`: `Bytes`

### Enum `DecodeLogError`

**Variants:**

- `UnexpectedTopic`
- `InvalidName`
- `DecodeError`
- `UnresolvedParent`

### Impl for `DecodeLogError`

#### `DecodeLogError::fmt` method

```rust
fn fmt (& self , f : & mut fmt :: Formatter < '_ >) -> fmt :: Result
```

### Impl for `DecodeLogError`

#### `valid_entry` function

```rust
fn valid_entry (entry : & str , note : bool , fact : bool) -> bool
```

#### `valid_name` function

```rust
fn valid_name (name : & str) -> bool
```

#### `valid_note` function

```rust
fn valid_note (note : & str) -> bool
```

#### `valid_fact` function

```rust
fn valid_fact (fact : & str) -> bool
```

#### `namehash` function

```rust
fn namehash (name : & str) -> String
```

#### `decode_mint_log` function

```rust
fn decode_mint_log (log : & crate :: eth :: Log) -> Result < Mint , DecodeLogError >
```

#### `decode_note_log` function

```rust
fn decode_note_log (log : & crate :: eth :: Log) -> Result < Note , DecodeLogError >
```

#### `decode_fact_log` function

```rust
fn decode_fact_log (log : & crate :: eth :: Log) -> Result < Fact , DecodeLogError >
```

#### `resolve_parent` function

```rust
fn resolve_parent (log : & crate :: eth :: Log , timeout : Option < u64 >) -> Option < String >
```

#### `resolve_full_name` function

```rust
fn resolve_full_name (log : & crate :: eth :: Log , timeout : Option < u64 >) -> Option < String >
```

### Struct `Hypermap`

**Fields:**

- `provider`: `Provider`

### Impl for `Hypermap`

#### `Hypermap::new` method

```rust
fn new (provider : Provider , address : Address) -> Self
```

#### `Hypermap::default` method

```rust
fn default (timeout : u64) -> Self
```

#### `Hypermap::address` method

```rust
fn address (& self) -> & Address
```

#### `Hypermap::get` method

```rust
fn get (& self , path : & str) -> Result < (Address , Address , Option < Bytes >) , EthError >
```

#### `Hypermap::get_hash` method

```rust
fn get_hash (& self , entryhash : & str) -> Result < (Address , Address , Option < Bytes >) , EthError >
```

#### `Hypermap::get_namehash_from_tba` method

```rust
fn get_namehash_from_tba (& self , tba : Address) -> Result < String , EthError >
```

#### `Hypermap::mint_filter` method

```rust
fn mint_filter (& self) -> crate :: eth :: Filter
```

#### `Hypermap::note_filter` method

```rust
fn note_filter (& self) -> crate :: eth :: Filter
```

#### `Hypermap::fact_filter` method

```rust
fn fact_filter (& self) -> crate :: eth :: Filter
```

#### `Hypermap::notes_filter` method

```rust
fn notes_filter (& self , notes : & [& str]) -> crate :: eth :: Filter
```

#### `Hypermap::facts_filter` method

```rust
fn facts_filter (& self , facts : & [& str]) -> crate :: eth :: Filter
```

## client.rs

### Enum `HttpClientAction`

**Variants:**

- `Http`
- `WebSocketOpen`
- `WebSocketPush`
- `WebSocketClose`

### Struct `OutgoingHttpRequest`

**Fields:**

- `method`: `String`
- `version`: `Option < String >`
- `url`: `String`
- `headers`: `HashMap < String , String >`

### Enum `HttpClientRequest`

**Variants:**

- `WebSocketPush`
- `WebSocketClose`

### Enum `HttpClientResponse`

**Variants:**

- `Http`
- `WebSocketAck`

### Enum `HttpClientError`

**Variants:**

- `MalformedRequest`
- `BadMethod`
- `BadUrl`
- `BadVersion`
- `BuildRequestFailed`
- `ExecuteRequestFailed`
- `WsOpenFailed`
- `WsPushUnknownChannel`
- `WsPushNoBlob`
- `WsPushBadText`
- `WsCloseFailed`

#### `send_request` function

```rust
fn send_request (method : Method , url : url :: Url , headers : Option < HashMap < String , String > > , timeout : Option < u64 > , body : Vec < u8 > ,)
```

#### `send_request_await_response` function

```rust
fn send_request_await_response (method : Method , url : url :: Url , headers : Option < HashMap < String , String > > , timeout : u64 , body : Vec < u8 > ,) -> std :: result :: Result < http :: Response < Vec < u8 > > , HttpClientError >
```

#### `open_ws_connection` function

```rust
fn open_ws_connection (url : String , headers : Option < HashMap < String , String > > , channel_id : u32 ,) -> std :: result :: Result < () , HttpClientError >
```

#### `send_ws_client_push` function

```rust
fn send_ws_client_push (channel_id : u32 , message_type : WsMessageType , blob : KiBlob)
```

#### `close_ws_connection` function

```rust
fn close_ws_connection (channel_id : u32) -> std :: result :: Result < () , HttpClientError >
```

## server.rs

### Enum `HttpServerRequest`

**Variants:**

- `Http`
- `WebSocketOpen`
- `WebSocketPush`
- `WebSocketClose`

### Impl for `HttpServerRequest`

#### `HttpServerRequest::from_bytes` method

```rust
fn from_bytes (bytes : & [u8]) -> serde_json :: Result < Self >
```

#### `HttpServerRequest::request` method

```rust
fn request (self) -> Option < IncomingHttpRequest >
```

### Struct `IncomingHttpRequest`

**Fields:**


### Impl for `IncomingHttpRequest`

#### `IncomingHttpRequest::url` method

```rust
fn url (& self) -> Result < url :: Url , url :: ParseError >
```

#### `IncomingHttpRequest::method` method

```rust
fn method (& self) -> Result < http :: Method , http :: method :: InvalidMethod >
```

#### `IncomingHttpRequest::source_socket_addr` method

```rust
fn source_socket_addr (& self) -> Result < std :: net :: SocketAddr , std :: net :: AddrParseError >
```

#### `IncomingHttpRequest::bound_path` method

```rust
fn bound_path (& self , process_id_to_strip : Option < & str >) -> & str
```

#### `IncomingHttpRequest::path` method

```rust
fn path (& self) -> Result < String , url :: ParseError >
```

#### `IncomingHttpRequest::headers` method

```rust
fn headers (& self) -> HeaderMap
```

#### `IncomingHttpRequest::url_params` method

```rust
fn url_params (& self) -> & HashMap < String , String >
```

#### `IncomingHttpRequest::query_params` method

```rust
fn query_params (& self) -> & HashMap < String , String >
```

### Enum `WsMessageType`

**Variants:**

- `Text`
- `Binary`
- `Ping`
- `Pong`
- `Close`

### Enum `HttpServerAction`

**Variants:**

- `Bind`
- `SecureBind`
- `Unbind`
- `WebSocketBind`
- `WebSocketSecureBind`
- `WebSocketUnbind`
- `WebSocketPush`
- `WebSocketExtPushOutgoing`
- `WebSocketExtPushData`
- `WebSocketClose`

### Struct `HttpResponse`

**Fields:**

- `status`: `u16`
- `headers`: `HashMap < String , String >`

### Impl for `HttpResponse`

#### `HttpResponse::new` method

```rust
fn new < T > (status : T) -> Self where T : Into < u16 > ,
```

#### `HttpResponse::set_status` method

```rust
fn set_status (mut self , status : u16) -> Self
```

#### `HttpResponse::header` method

```rust
fn header < T , U > (mut self , key : T , value : U) -> Self where T : Into < String > , U : Into < String > ,
```

#### `HttpResponse::set_headers` method

```rust
fn set_headers (mut self , headers : HashMap < String , String >) -> Self
```

### Enum `HttpServerError`

**Variants:**

- `MalformedRequest`
- `NoBlob`
- `InvalidSourceProcess`
- `WsPingPongTooLong`
- `WsChannelNotFound`
- `Timeout`
- `UnexpectedResponse`

### Enum `MessageType`

**Variants:**

- `Request`
- `Response`

### Struct `HttpServer`

**Fields:**

- `timeout`: `u64`

### Struct `HttpBindingConfig`

**Fields:**


### Impl for `HttpBindingConfig`

#### `HttpBindingConfig::default` method

```rust
fn default () -> Self
```

#### `HttpBindingConfig::new` method

```rust
fn new (authenticated : bool , local_only : bool , secure_subdomain : bool , static_content : Option < KiBlob > ,) -> Self
```

#### `HttpBindingConfig::authenticated` method

```rust
fn authenticated (mut self , authenticated : bool) -> Self
```

#### `HttpBindingConfig::local_only` method

```rust
fn local_only (mut self , local_only : bool) -> Self
```

#### `HttpBindingConfig::secure_subdomain` method

```rust
fn secure_subdomain (mut self , secure_subdomain : bool) -> Self
```

#### `HttpBindingConfig::static_content` method

```rust
fn static_content (mut self , static_content : Option < KiBlob >) -> Self
```

### Struct `WsBindingConfig`

**Fields:**


### Impl for `WsBindingConfig`

#### `WsBindingConfig::default` method

```rust
fn default () -> Self
```

#### `WsBindingConfig::new` method

```rust
fn new (authenticated : bool , secure_subdomain : bool , extension : bool) -> Self
```

#### `WsBindingConfig::authenticated` method

```rust
fn authenticated (mut self , authenticated : bool) -> Self
```

#### `WsBindingConfig::secure_subdomain` method

```rust
fn secure_subdomain (mut self , secure_subdomain : bool) -> Self
```

#### `WsBindingConfig::extension` method

```rust
fn extension (mut self , extension : bool) -> Self
```

### Impl for `HttpServer`

#### `HttpServer::new` method

```rust
fn new (timeout : u64) -> Self
```

#### `HttpServer::bind_http_path` method

```rust
fn bind_http_path < T > (& mut self , path : T , config : HttpBindingConfig ,) -> Result < () , HttpServerError > where T : Into < String > ,
```

#### `HttpServer::bind_ws_path` method

```rust
fn bind_ws_path < T > (& mut self , path : T , config : WsBindingConfig ,) -> Result < () , HttpServerError > where T : Into < String > ,
```

#### `HttpServer::bind_http_static_path` method

```rust
fn bind_http_static_path < T > (& mut self , path : T , authenticated : bool , local_only : bool , content_type : Option < String > , content : Vec < u8 > ,) -> Result < () , HttpServerError > where T : Into < String > ,
```

#### `HttpServer::secure_bind_http_path` method

```rust
fn secure_bind_http_path < T > (& mut self , path : T) -> Result < () , HttpServerError > where T : Into < String > ,
```

#### `HttpServer::secure_bind_ws_path` method

```rust
fn secure_bind_ws_path < T > (& mut self , path : T) -> Result < () , HttpServerError > where T : Into < String > ,
```

#### `HttpServer::modify_http_path` method

```rust
fn modify_http_path < T > (& mut self , path : & str , config : HttpBindingConfig ,) -> Result < () , HttpServerError > where T : Into < String > ,
```

#### `HttpServer::modify_ws_path` method

```rust
fn modify_ws_path (& mut self , path : & str , config : WsBindingConfig ,) -> Result < () , HttpServerError >
```

#### `HttpServer::unbind_http_path` method

```rust
fn unbind_http_path < T > (& mut self , path : T) -> Result < () , HttpServerError > where T : Into < String > ,
```

#### `HttpServer::unbind_ws_path` method

```rust
fn unbind_ws_path < T > (& mut self , path : T) -> Result < () , HttpServerError > where T : Into < String > ,
```

#### `HttpServer::serve_file` method

```rust
fn serve_file (& mut self , file_path : & str , paths : Vec < & str > , config : HttpBindingConfig ,) -> Result < () , HttpServerError >
```

#### `HttpServer::serve_file_raw_path` method

```rust
fn serve_file_raw_path (& mut self , file_path : & str , paths : Vec < & str > , config : HttpBindingConfig ,) -> Result < () , HttpServerError >
```

#### `HttpServer::serve_ui` method

```rust
fn serve_ui (& mut self , directory : & str , roots : Vec < & str > , config : HttpBindingConfig ,) -> Result < () , HttpServerError >
```

#### `HttpServer::handle_websocket_open` method

```rust
fn handle_websocket_open (& mut self , path : & str , channel_id : u32)
```

#### `HttpServer::handle_websocket_close` method

```rust
fn handle_websocket_close (& mut self , channel_id : u32)
```

#### `HttpServer::parse_request` method

```rust
fn parse_request (& self , body : & [u8]) -> Result < HttpServerRequest , HttpServerError >
```

#### `HttpServer::handle_request` method

```rust
fn handle_request (& mut self , server_request : HttpServerRequest , mut http_handler : impl FnMut (IncomingHttpRequest) -> (HttpResponse , Option < KiBlob >) , mut ws_handler : impl FnMut (u32 , WsMessageType , KiBlob) ,)
```

#### `HttpServer::ws_push_all_channels` method

```rust
fn ws_push_all_channels (& self , path : & str , message_type : WsMessageType , blob : KiBlob)
```

#### `HttpServer::get_ws_channels` method

```rust
fn get_ws_channels (& self) -> HashMap < String , HashSet < u32 > >
```

#### `HttpServer::bind_multiple_http_paths` method

```rust
fn bind_multiple_http_paths < T : Into < String > > (& mut self , paths : Vec < T > , config : HttpBindingConfig ,) -> Result < () , HttpServerError >
```

#### `send_response` function

```rust
fn send_response (status : StatusCode , headers : Option < HashMap < String , String > > , body : Vec < u8 >)
```

#### `send_ws_push` function

```rust
fn send_ws_push (channel_id : u32 , message_type : WsMessageType , blob : KiBlob)
```

#### `ws_push_all_channels` function

```rust
fn ws_push_all_channels (ws_channels : & HashMap < String , HashSet < u32 > > , path : & str , message_type : WsMessageType , blob : KiBlob ,)
```

#### `get_mime_type` function

```rust
fn get_mime_type (filename : & str) -> String
```

## mod.rs

## kernel_types.rs

### Struct `LazyLoadBlob`

**Fields:**

- `mime`: `Option < String >`
- `bytes`: `Vec < u8 >`

### Struct `Request`

**Fields:**

- `inherit`: `bool`
- `expects_response`: `Option < u64 >`
- `body`: `Vec < u8 >`
- `metadata`: `Option < String >`
- `capabilities`: `Vec < Capability >`

### Struct `Response`

**Fields:**

- `inherit`: `bool`
- `body`: `Vec < u8 >`
- `metadata`: `Option < String >`
- `capabilities`: `Vec < Capability >`

### Enum `Message`

**Variants:**

- `Request`
- `Response`

### Struct `Capability`

**Fields:**

- `issuer`: `Address`
- `params`: `String`

### Impl for `Capability`

### Impl for `Capability`

#### `Capability::eq` method

```rust
fn eq (& self , other : & Self) -> bool
```

### Impl for `Capability`

#### `Capability::hash` method

```rust
fn hash < H : Hasher > (& self , state : & mut H)
```

### Impl for `Capability`

#### `Capability::new` method

```rust
fn new < T , U > (issuer : T , params : U) -> Self where T : Into < Address > , U : Into < String > ,
```

#### `Capability::messaging` method

```rust
fn messaging < T > (issuer : T) -> Self where T : Into < Address > ,
```

### Impl for `Capability`

#### `Capability::fmt` method

```rust
fn fmt (& self , f : & mut std :: fmt :: Formatter < '_ >) -> std :: fmt :: Result
```

### Struct `SendError`

**Fields:**

- `kind`: `SendErrorKind`
- `target`: `Address`
- `message`: `Message`
- `lazy_load_blob`: `Option < LazyLoadBlob >`

### Enum `SendErrorKind`

**Variants:**

- `Offline`
- `Timeout`

### Enum `OnExit`

**Variants:**

- `None`
- `Restart`
- `Requests`

### Impl for `OnExit`

#### `OnExit::is_restart` method

```rust
fn is_restart (& self) -> bool
```

### Enum `KernelCommand`

**Variants:**

- `Booted`
- `InitializeProcess`
- `GrantCapabilities`
- `DropCapabilities`
- `RunProcess`
- `KillProcess`
- `Shutdown`
- `Debug`

### Enum `KernelPrint`

**Variants:**

- `ProcessMap`
- `Process`
- `HasCap`

### Enum `KernelResponse`

**Variants:**

- `InitializedProcess`
- `InitializeProcessError`
- `StartedProcess`
- `RunProcessError`
- `KilledProcess`
- `Debug`

### Enum `KernelPrintResponse`

**Variants:**

- `ProcessMap`
- `Process`
- `HasCap`

### Struct `PersistedProcess`

**Fields:**

- `wasm_bytes_handle`: `String`
- `wit_version`: `Option < u32 >`
- `on_exit`: `OnExit`
- `capabilities`: `HashSet < Capability >`
- `public`: `bool`

### Enum `StateAction`

**Variants:**

- `GetState`
- `SetState`
- `DeleteState`
- `Backup`

### Enum `StateResponse`

**Variants:**

- `GetState`
- `SetState`
- `DeleteState`
- `Backup`
- `Err`

### Enum `StateError`

**Variants:**

- `RocksDBError`
- `StartupError`
- `BadBytes`
- `BadRequest`
- `BadJson`
- `NotFound`
- `IOError`

### Impl for `StateError`

#### `StateError::kind` method

```rust
fn kind (& self) -> & str
```

### Struct `Erc721Metadata`

**Fields:**

- `name`: `Option < String >`
- `description`: `Option < String >`
- `image`: `Option < String >`
- `external_url`: `Option < String >`
- `animation_url`: `Option < String >`
- `properties`: `Erc721Properties`

### Struct `Erc721Properties`

**Fields:**

- `package_name`: `String`
- `publisher`: `String`
- `current_version`: `String`
- `mirrors`: `Vec < NodeId >`
- `code_hashes`: `HashMap < String , String >`
- `license`: `Option < String >`
- `screenshots`: `Option < Vec < String > >`
- `wit_version`: `Option < u32 >`
- `dependencies`: `Option < Vec < String > >`
- `api_includes`: `Option < Vec < std :: path :: PathBuf > >`

### Struct `PackageManifestEntry`

**Fields:**

- `process_name`: `String`
- `process_wasm_path`: `String`
- `on_exit`: `OnExit`
- `request_networking`: `bool`
- `request_capabilities`: `Vec < serde_json :: Value >`
- `grant_capabilities`: `Vec < serde_json :: Value >`
- `public`: `bool`

### Struct `DotScriptsEntry`

**Fields:**

- `root`: `bool`
- `public`: `bool`
- `request_networking`: `bool`
- `request_capabilities`: `Option < Vec < serde_json :: Value > >`
- `grant_capabilities`: `Option < Vec < serde_json :: Value > >`
- `wit_version`: `Option < u32 >`

### Impl for `Message`

#### `Message::fmt` method

```rust
fn fmt (& self , f : & mut std :: fmt :: Formatter) -> std :: fmt :: Result
```

#### `de_wit_address` function

```rust
fn de_wit_address (wit : wit :: Address) -> Address
```

#### `en_wit_address` function

```rust
fn en_wit_address (address : Address) -> wit :: Address
```

#### `de_wit_request` function

```rust
fn de_wit_request (wit : wit :: Request) -> Request
```

#### `en_wit_request` function

```rust
fn en_wit_request (request : Request) -> wit :: Request
```

#### `de_wit_response` function

```rust
fn de_wit_response (wit : wit :: Response) -> Response
```

#### `en_wit_response` function

```rust
fn en_wit_response (response : Response) -> wit :: Response
```

#### `de_wit_blob` function

```rust
fn de_wit_blob (wit : Option < wit :: LazyLoadBlob >) -> Option < LazyLoadBlob >
```

#### `en_wit_blob` function

```rust
fn en_wit_blob (load : Option < LazyLoadBlob >) -> Option < wit :: LazyLoadBlob >
```

#### `de_wit_capability` function

```rust
fn de_wit_capability (wit : wit :: Capability) -> Capability
```

#### `en_wit_capability` function

```rust
fn en_wit_capability (cap : Capability) -> wit :: Capability
```

#### `en_wit_message` function

```rust
fn en_wit_message (message : Message) -> wit :: Message
```

#### `en_wit_send_error` function

```rust
fn en_wit_send_error (error : SendError) -> wit :: SendError
```

#### `en_wit_send_error_kind` function

```rust
fn en_wit_send_error_kind (kind : SendErrorKind) -> wit :: SendErrorKind
```

## homepage.rs

#### `add_to_homepage` function

```rust
fn add_to_homepage (label : & str , icon : Option < & str > , path : Option < & str > , widget : Option < & str >)
```

#### `remove_from_homepage` function

```rust
fn remove_from_homepage ()
```

