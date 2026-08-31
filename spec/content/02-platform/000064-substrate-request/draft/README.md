# XTalk substrate requests

This draft mirrors `base-request`, the request helpers in `base-util`, and the
root receive functions. It intentionally omits timeout, cancellation, owner,
deadline, and shutdown semantics because the pinned XTalk implementation does
not define them.
