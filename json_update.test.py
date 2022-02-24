import json_update as ju
import json as jsn

# template = {
#     "a": 1,
#     "b": 2,
#     "c": 3,
#     "d": {
#         "da": 10,
#         "db": 11
#     }
# }

# comp = {
#     "a": 111,
#     "d": {
#         "db" :1
#     }
# }

# template = ju.read_json('.configs/botsettings.template.json')
# comp = ju.read_json('.configs/botsettings.debug.json')


# def display(obj):
#     print(jsn.dumps(obj, indent=4), end='\n\n')


# display(template)
# display(comp)
# ju.recursive_update(template, comp)
# display(comp)

# result = {}

# ju.recursive_reorder(template, comp, result)

# display(result)

# print(jsn.dumps(template))
# print(jsn.dumps(comp))
# print(jsn.dumps(ju.recursive_update(template,comp)))