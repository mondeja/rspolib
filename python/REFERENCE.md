# rspolib Python bindings reference

- [Exceptions](#exceptions)
  - [`IOError`](#ioerror)
  - [`SyntaxError`](#syntaxerror)
  - [`EscapingError`](#escapingerror)
- [Factories](#factories)
  - [`pofile(...) -> POFile`](#pofilepath_or_content-str-wrapwidth-int--78---pofile)
  - [`mofile(...) -> MOFile`](#mofilepath_or_content-str-wrapwidth-int--78---mofile)
- [Classes](#classes)
  - [`POFile`](#pofile)
  - [`MOFile`](#mofile)
  - [`POEntry`](#poentry)
  - [`MOEntry`](#moentry)
- [Utilities](#utilities)
  - [`unescape(...) -> str`](#unescapestring-str---str)
  - [`escape(...) -> str`](#escapestring-str---str)

## Exceptions

### `IOError`

Raised when there is an error parsing the content of a MO file.

### `SyntaxError`

Raised when there is an error parsing the content of a PO file.

### `EscapingError`

Raised when there is an error unescaping a string.

This error is not directly raised by the parsing functions, only is useful when you are calling [`unescape`](#unescapestring-str---str).

## Factories

### `pofile(path_or_content: str, wrapwidth: int = 78) -> POFile`

Parse a PO file by path or content and returns a [`POFile`](#pofile) object.

Can raise a [`SyntaxError`](#syntaxerror) if a problem is found parsing the file.

### `mofile(path_or_content: str, wrapwidth: int = 78) -> MOFile`

Parse a MO file by path or content and returns a [`MOFile`](#mofile) object.

Can raise an [`IOError`](#ioerror) if a problem is found parsing the file.

## Classes

### `POFile`

#### `__init__(self, path_or_content: str = "", wrapwidth: int = 78)`

Create a new [`POFile`](#pofile) object, by default empty.

#### `get_entries(self) -> List[POEntry]`

Returns a copy of the entries of the file.

#### `@setter entries(self, entries: List[POEntry])`

Sets the entries of the file.

```python
from rspolib import POFile, POEntry

po = POFile()
po.entries = [
    POEntry(msgid="Hello", msgstr="Hola"),
    POEntry(msgid="Goodbye", msgstr="Adiós"),
]
```

#### `@getter header(self) -> str`

Returns a copy of the header of the file.

#### `@setter header(self, header: str)`

Sets the header of the file.

#### `get_metadata(self) -> Dict[str, str]`

Returns a copy of the metadata of the file.

#### `@setter metadata(self, metadata: Dict[str, str])`

Sets the metadata of the file.

#### `update_metadata(self, metadata: Dict[str, str])`

Updates the metadata of the file.

The previous metadata fields will be overwritten by the new one.

#### `remove_metadata_field(self, key: str)`

Removes a metadata field from the file.

#### `@getter metadata_is_fuzzy(self) -> bool`

Returns whether the metadata is fuzzy.

#### `@setter metadata_is_fuzzy(self, is_fuzzy: bool)`

Sets whether the metadata is fuzzy.

#### `save(self, path: str)`

#### `save_as_pofile(self, path: str)`

#### `save_as_mofile(self, path: str)`

Save the file to a path.

#### `remove(self, entry: POEntry)`

Removes an entry from the file.

#### `remove_by_msgid(self, msgid: str)`

Removes an entry from the file by its `msgid`.

#### `remove_by_msgid_msgctxt(self, msgid: str, msgctxt: str)`

Removes an entry from the file by its `msgid` and `msgctxt`.

#### `append(self, entry: POEntry)`

Appends an entry to the file.

#### `find(value: str, by: str = "msgid", include_obsolete_entries: bool = False, msgctxt: Optional[str] = None) -> List[POEntry]`

Finds entries by a given value.

- `value`: The value to search.
- `by`: The field to search. Can be `msgid`, `msgstr`, `msgctxt`, `msgid_plural`, `previous_msgid`, `previous_msgid_plural` or `previous_msgctxt`.
- `include_obsolete_entries`: Whether to include obsolete entries in the search.
- `msgctxt`: The context to match against too while searching. Only used when `by` is not `msgctxt`.

> If you are doing a search by `msgid` or `msgid` + `msgctxt` the functions `find_by_msgid` and `find_by_msgid_msgctxt` are more efficient.

#### `find_by_msgid(self, msgid: str) -> Optional[POEntry]`

Finds an entry by its `msgid`.

#### `find_by_msgid_msgctxt(self, msgid: str, msgctxt: str) -> Optional[POEntry]`

Finds an entry by its `msgid` and `msgctxt`.

#### `percent_translated(self) -> float`

Returns the percentage of translated entries.

#### `translated_entries(self) -> List[POEntry]`

Returns a list with copies of the translated entries.

#### `untranslated_entries(self) -> List[POEntry]`

Returns a list with copies of the untranslated entries.

#### `obsolete_entries(self) -> List[POEntry]`

Returns a list with copies of the obsolete entries.

#### `fuzzy_entries(self) -> List[POEntry]`

Returns a list with copies of the fuzzy entries.

#### `__len__(self) -> int`

Returns the number of entries in the file.

#### `__contains__(self, entry: POEntry) -> bool`

Returns whether the file contains an entry.

```python
from rspolib import POFile, POEntry

file = POFile()
entry = POEntry(msgid="Hello", msgstr="Hola")
file.append(entry)
assert entry in file
```

#### `__getitem__(self, index: int) -> POEntry`

Returns an entry from the file.

```python
from rspolib import POFile, POEntry

file = POFile()
entry = POEntry(msgid="Hello")
file.append(entry)
assert file[0].msgid == "Hello"
```

#### `__str__(self) -> str`

Returns the content of the file as a string.

#### `__eq__(self, other: POFile) -> bool`

#### `__ne__(self, other: POFile) -> bool`

Returns whether two files are equal or not comparing by their produced contents.

#### `__iter__(self) -> Iterator[POEntry]`

Returns an iterator over the entries of the file.

```python
from rspolib import POFile, POEntry

file = POFile()
file.append(POEntry(msgid="Hello"))
file.append(POEntry(msgid="Goodbye"))

for entry in file:
    print(entry.msgid)
```

### `MOFile`

#### `MAGIC: int`

#### `MAGIC_SWAPPED: int`

Possible magic numbers for MO files.

Indicate that the bytes of the file must be read in little endian (`MAGIC`) or big endian byte order (`MAGIC_SWAPPED`).

#### `__init__(self, path_or_content: str = "", wrapwidth: int = 78)`

Creates a new [`MOFile`](#mofile) object, by default empty.

#### `@getter magic_number(self) -> int`

Returns the magic number of the file.

#### `@getter version(self) -> int`

Returns the revision number of the file.

#### `get_entries(self) -> List[MOEntry]`

Returns a list with copies of the entries of the file.

#### `@setter entries(self, entries: List[MOEntry])`

Sets the entries of the file.

```python
from rspolib import MOFile, MOEntry

file = MOFile()
file.entries = [
    MOEntry(msgid="Hello", msgstr="Hola"),
    MOEntry(msgid="Goodbye", msgstr="Adiós"),
]
```

#### `get_metadata(self) -> Dict[str, str]`

Returns a copy of the metadata of the file.

#### `@setter metadata(self, metadata: Dict[str, str])`

Sets the metadata of the file.

#### `update_metadata(self, metadata: Dict[str, str])`

Updates the metadata of the file.

The previous metadata fields will be overwritten by the new one.

#### `remove_metadata_field(self, key: str)`

Removes a metadata field from the file.

#### `save(self, path: str)`

#### `save_as_pofile(self, path: str)`

#### `save_as_mofile(self, path: str)`

Save the file to a path.

#### `remove(self, entry: MOEntry)`

Removes an entry from the file.

#### `remove_by_msgid(self, msgid: str)`

Removes an entry from the file by its `msgid`.

#### `remove_by_msgid_msgctxt(self, msgid: str, msgctxt: str)`

Removes an entry from the file by its `msgid` and `msgctxt`.

#### `append(self, entry: MOEntry)`

Appends an entry to the file.

#### `find(value: str, by: str = "msgid", include_obsolete_entries: bool = False, msgctxt: Optional[str] = None) -> List[MOEntry]`

Finds entries by a given value.

- `value`: The value to search.
- `by`: The field to search. Can be `msgid`, `msgstr`, `msgctxt`, `msgid_plural`, `previous_msgid`, `previous_msgid_plural` or `previous_msgctxt`.
- `include_obsolete_entries`: Whether to include obsolete entries in the search.
- `msgctxt`: The context to match against too while searching. Only used when `by` is not `msgctxt`.

> If you are doing a search by `msgid` or `msgid` + `msgctxt` the functions `find_by_msgid` and `find_by_msgid_msgctxt` are more efficient.

#### `find_by_msgid(self, msgid: str) -> Optional[MOEntry]`

Finds an entry by its `msgid`.

#### `find_by_msgid_msgctxt(self, msgid: str, msgctxt: str) -> Optional[MOEntry]`

Finds an entry by its `msgid` and `msgctxt`.

#### `__len__(self) -> int`

Returns the number of entries in the file.

#### `__contains__(self, entry: MOEntry) -> bool`

Returns whether the file contains an entry.

#### `__getitem__(self, index: int) -> MOEntry`

Returns an entry from the file.

#### `__str__(self) -> str`

Returns the content of the file as a string.

#### `__eq__(self, other: POFile) -> bool`

#### `__ne__(self, other: POFile) -> bool`

Returns whether two files are equal or not comparing by their produced contents.

#### `__iter__(self) -> Iterator[MOEntry]`

Returns an iterator over the entries of the file.

### `POEntry`

#### `__init__(self, ...)`

- `msgid: str = ""`: The singular form of the message.
- `msgstr: Optional[str] = None`: The translation of the message.
- `msgid_plural: Optional[str] = None`: The plural form of the message.
- `msgstr_plural: List[str] = []`: The translations of the plural of the message.
- `msgctxt: Optional[str] = None`: The context of the message.
- `tcomment: Optional[str] = None`: A generated comment for translators.
- `comment: Optional[str] = None`: A generated comment for machines.
- `flags: List[str] = []`: A list of flags.

#### `@getter msgid(self) -> str`

#### `@setter msgid(self, msgid: str)`

#### `@getter msgstr(self) -> Optional[str]`

#### `@setter msgstr(self, msgstr: Optional[str])`

#### `@getter msgid_plural(self) -> Optional[str]`

#### `@setter msgid_plural(self, msgid_plural: Optional[str])`

#### `get_msgstr_plural(self) -> List[str]`

#### `@setter msgstr_plural(self, msgstr_plural: List[str])`

#### `@getter msgctxt(self) -> Optional[str]`

#### `@setter msgctxt(self, msgctxt: Optional[str])`

#### `@getter obsolete(self) -> bool`

#### `@setter obsolete(self, obsolete: bool)`

#### `@getter tcomment(self) -> Optional[str]`

#### `@setter tcomment(self, tcomment: Optional[str])`

#### `@getter comment(self) -> Optional[str]`

#### `@setter comment(self, comment: Optional[str])`

#### `get_flags(self) -> List[str]`

#### `@setter flags(self, flags: List[str])`

#### `get_occurrences(self) -> List[(str, str)]`

#### `@setter occurrences(self, occurrences: List[(str, str)]`

#### `@getter previous_msgid(self) -> Optional[str]`

#### `@setter previous_msgid(self, previous_msgid: Optional[str])`

#### `@getter previous_msgid_plural(self) -> Optional[str]`

#### `@setter previous_msgid_plural(self, previous_msgid_plural: Optional[str])`

#### `@getter previous_msgctxt(self) -> Optional[str]`

#### `@setter previous_msgctxt(self, previous_msgctxt: Optional[str])`

#### `@getter linenum(self) -> int`

Returns the line number of the entry in the file.

#### `@getter fuzzy(self) -> bool`

Returns whether the entry is fuzzy.

Convenient way to check `"fuzzy" in entry.flags`.

#### `translated(self) -> bool`

Returns whether the entry is translated.

#### `merge(self, other: POEntry)`

Merges the entry with another one.

#### `__str__(self) -> str`

#### `__eq__(self, other: POEntry) -> bool`

#### `__ne__(self, other: POEntry) -> bool`

#### `__gt__(self, other: POEntry) -> bool`

#### `__ge__(self, other: POEntry) -> bool`

#### `__lt__(self, other: POEntry) -> bool`

#### `__le__(self, other: POEntry) -> bool`

#### `__cmp__(self, other: POEntry) -> int`

### `MOEntry`

#### `__init__(self, ...)`

- `msgid: str = ""`: The singular form of the message.
- `msgstr: Optional[str] = None`: The translation of the message.
- `msgid_plural: Optional[str] = None`: The plural form of the message.
- `msgstr_plural: List[str] = []`: The translations of the plural of the message.
- `msgctxt: Optional[str] = None`: The context of the message.

#### `@getter msgid(self) -> str`

#### `@setter msgid(self, msgid: str)`

#### `@getter msgstr(self) -> Optional[str]`

#### `@setter msgstr(self, msgstr: Optional[str])`

#### `@getter msgid_plural(self) -> Optional[str]`

#### `@setter msgid_plural(self, msgid_plural: Optional[str])`

#### `get_msgstr_plural(self) -> List[str]`

#### `@setter msgstr_plural(self, msgstr_plural: List[str])`

#### `@getter msgctxt(self) -> Optional[str]`

#### `@setter msgctxt(self, msgctxt: Optional[str])`

#### `translated(self) -> bool`

Returns whether the entry is translated.

#### `merge(self, other: POEntry)`

Merges the entry with another one.

#### `__str__(self) -> str`

#### `__eq__(self, other: POEntry) -> bool`

#### `__ne__(self, other: POEntry) -> bool`

#### `__gt__(self, other: POEntry) -> bool`

#### `__ge__(self, other: POEntry) -> bool`

#### `__lt__(self, other: POEntry) -> bool`

#### `__le__(self, other: POEntry) -> bool`

#### `__cmp__(self, other: POEntry) -> int`

## Utilities

### `unescape(text: str) -> str`

Unescapes a PO file string.

Can raise an [`EscapingError`](#escapingerror) if one of the next problems is found:

- A escape character is escaping a character that should not be escaped.
- There is an escape character at the end of the string.

### `escape(text: str) -> str`

Escapes a PO file string.
