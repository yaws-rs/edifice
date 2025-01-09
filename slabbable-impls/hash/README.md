# Slabbable Hash

Hash impl Slabbable

Currently we only hook hashbrown as HashMap and nohash-hasher as the HasherBuilder

If you would like other hasher / hashmap please add with cfg-switches;

| cfg               | values | default       |
| :---              | :---   | :---          |
| slabbable_hasher  | -      | nohash_hasher |
| slabbable_hashmap | -      | hashbrown     |
