def remap(input, output, file=None):
    import json
    print('[[remap]]', file=file)
    print(f'input = {json.dumps(input)}', file=file)
    print(f'output = {json.dumps(output)}', file=file)
