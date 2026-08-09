# Play and reshape Glass Signal

This complete Showcase project requests the narrow `:audio/playback` capability.
Loading the graph is still silent: open the Audio surface and press **Play** once
to authorize browser audio.

After authorization, evaluate any of these forms independently:

```clojure
(sonic/update "showcase/supersonic-glass-signal" "transport" "tempo" 138)
(sonic/update "showcase/supersonic-glass-signal" "source" "waveform" "saw")
(sonic/update "showcase/supersonic-glass-signal" "source" "root" 55)
(sonic/update "showcase/supersonic-glass-signal" "mixer" "volume" 0.45)
```

Typed updates preserve the current graph overlay and sequencer clock. Stop closes
the browser resources and revokes playback authorization.
