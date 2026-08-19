#!/usr/bin/env python3
from pathlib import Path

core = Path('core/lib/src/lang/core.hal')
source = core.read_text()

old = '     (intern-var (current-namespace) declaration-symbol pointer))))'
if source.count(old) != 1:
    raise SystemExit(f'code-pointer publication anchor changed: {source.count(old)}')
source = source.replace(old, '     pointer)))', 1)

old = '    (intern-var (current-namespace) declaration-symbol pointer)))'
if source.count(old) != 1:
    raise SystemExit(f'fragment-pointer publication anchor changed: {source.count(old)}')
source = source.replace(old, '    pointer))', 1)

old = '''    (list
     'defmacro
     macro-symbol
     (vector 'symbol '& 'body)
     (list
      'list
      (list 'quote 'lang.core/register-code-pointer!)
      lang
      (list
       'list
       (list 'quote 'quote)
       (list
        'apply
        'list
        (list 'quote operation)
        'symbol
        'body))))))'''
new = '''    (list
     'defmacro
     macro-symbol
     (vector 'symbol '& 'body)
     (list
      'list
      (list 'quote 'def)
      'symbol
      (list
       'list
       (list 'quote 'lang.core/register-code-pointer!)
       lang
       (list
        'list
        (list 'quote 'quote)
        (list
         'apply
         'list
         (list 'quote operation)
         'symbol
         'body)))))))'''
if source.count(old) != 1:
    raise SystemExit(f'definition macro publication anchor changed: {source.count(old)}')
source = source.replace(old, new, 1)

old = '''    (list
     'defmacro
     macro-symbol
     (vector 'symbol 'value)
     (list
      'list
      (list 'quote 'lang.core/register-fragment-pointer!)
      lang
      (list 'list (list 'quote 'quote) 'symbol)
      (list 'list (list 'quote 'quote) 'value)
      (list
       'list
       (list 'quote 'quote)
       (list 'or (list 'meta 'symbol) {}))))))'''
new = '''    (list
     'defmacro
     macro-symbol
     (vector 'symbol 'value)
     (list
      'list
      (list 'quote 'def)
      'symbol
      (list
       'list
       (list 'quote 'lang.core/register-fragment-pointer!)
       lang
       (list 'list (list 'quote 'quote) 'symbol)
       (list 'list (list 'quote 'quote) 'value)
       (list
        'list
        (list 'quote 'quote)
        (list 'or (list 'meta 'symbol) {})))))))'''
if source.count(old) != 1:
    raise SystemExit(f'fragment macro publication anchor changed: {source.count(old)}')
source = source.replace(old, new, 1)
core.write_text(source.rstrip() + '\n')

test = Path('core/lib/test-lang/lang/core/script_port_test.hal')
text = test.read_text()
old = "(script-lint/collect-vars '#{[hello #{world}]})"
new = "(sort (map str (script-lint/collect-vars\n                       #{[(symbol \"hello\") #{(symbol \"world\")}]})))"
if text.count(old) != 1:
    raise SystemExit('script-lint test anchor changed')
text = text.replace(old, new, 1)
text = text.replace("=> [#{hello world} true [:tool.lint/unresolved-symbol]])",
                    "=> [[\"hello\" \"world\"] true [:tool.lint/unresolved-symbol]])", 1)
text = text.replace('(map? stopped) after])', '(not (nil? stopped)) after])', 1)
test.write_text(text)
