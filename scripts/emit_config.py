def remap(cond, exc, when, mappings, file=None):
    import json
    print('[[remap]]', file=file)
    print(f'cond = {json.dumps(_to_list(cond))}', file=file)
    print(f'except = {json.dumps(_to_list(exc))}', file=file)
    print(f'when = {json.dumps(_to_list(when))}', file=file)
    print('[remap.mappings]', file=file)
    for k, v in mappings.items():
        print(f'{k} = {json.dumps(_to_list(v))}', file=file)


def _to_list(x):
    if type(x) == list:
        return x
    return [x]
