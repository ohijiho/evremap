#!/usr/bin/env python3
import sys
import json
import yaml
root = yaml.safe_load(sys.stdin)
assert type(root) == dict
assert root.keys() == {'remap'}
remap = root['remap']
assert type(remap) == dict
for k, v in remap.items():
    print('[[cond_remap]]')
    print(f'key = "{k}"')
    assert type(v) == list
    for rule in v:
        assert type(rule) == dict
        assert {*rule.keys()} <= {'cond', 'keys'}
        cond = rule.get('cond')
        assert type(cond) in {type(None), list, str}
        if cond is None:
            cond = [[]]
        elif type(cond) == str:
            cond = [[cond]]
        for x in cond:
            assert type(x) in {list, str}
            if type(x) == list:
                assert all(type(y) == str for y in x)
        cond = [x if type(x) == list else [x] for x in cond]
        keys = rule.get('keys')
        assert type(keys) in {type(None), list, str}
        if keys is None:
            keys = []
        elif type(keys) == str:
            keys = [keys]
        assert all(type(x) == str for x in keys)
        print('[[cond_remap.rule]]')
        print(f'cond = {json.dumps(cond)}')
        print(f'keys = {json.dumps(keys)}')
