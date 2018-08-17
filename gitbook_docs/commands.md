# List of Commands

## SET &lt;key> &lt;value>

Store string `value` that can be retrieved by `key`.

#### Example

```
> SET hello world
OK
> GET hello
"world"
```

## GET &lt;key>

Retrieve the string value of the `key`.

#### Example

```
> SET hello world
OK
> GET hello
"world"
> GET nonexistent
(error) ERROR: Key not found
```

## DELETE &lt;key>

Remove the `key` and its string value from the key-value store.

#### Example

```
> SET hello world
OK
> DELETE hello
OK
> GET hello
(error) ERROR: Key not found
> DELETE nonexistent
(error) ERROR: Key not found
```

## EXISTS &lt;key>

Checks for the existence of `key` in the key-value store. Returns 1 if it exists, 0 otherwise.

#### Example

```
> EXISTS nonexistent
(integer) 0
> SET hello world
OK
> EXISTS hello
(integer) 1
```

## DESTROY

Removes all key-value pairs.

#### Example

```
> SET hello world
OK
> SET foo bar
OK
> DESTROY
OK
> GET hello
(error) ERROR: Key not found
> GET foo
(error) ERROR: Key not found
```

## QUIT/EXIT

Close the TCP connection between client and server.

#### Example

```
> QUIT
OK
```
