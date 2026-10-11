def call(method, *args):
    """Calls a native method, retrying with lone surrogates replaced.

    Python strings may hold lone surrogates (for example after decoding with
    ``surrogateescape``), which cannot be passed to Rust. ROUGE treats them as
    separators, so replacing them with ``?`` gives the same tokens.
    """
    try:
        return method(*args)
    except UnicodeEncodeError:
        return method(*(_encodable(arg) for arg in args))


def _encodable(value):
    if isinstance(value, str):
        return value.encode("utf-8", "replace").decode("utf-8")
    if isinstance(value, list):
        return [_encodable(item) for item in value]
    return value
