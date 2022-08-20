'''Wikilookup uses the Wikipedia Opensearch API to get articles.
See more here: https://www.mediawiki.org/wiki/API:Main_page
'''

from ipaddress import summarize_address_range
import requests as rq
import json
import random

WIKI_API_URL: str = "https://en.wikipedia.org/w/api.php"
RANDOM_LIMIT: int = 10

def get_search_result(query: str, position: int = 1) -> str:
    '''Gets a single article name for a particular query string.
    Specify the position of the ranked articles, to get a particular article title
    '''

    resp = rq.get(
        url = WIKI_API_URL,
        params = {
            "action": "opensearch",
            "namespace": "0",
            "search" : query,
            "limit": position,
            "format": "json"
        }
    )

    result_arr = json.loads(resp.text)[1]
    if len(result_arr) < position:
        return result_arr[-1]
    else:
        return result_arr[position-1]


def get_article_summary(article_name: str) -> str:
    '''Gets the first section of the article, AKA the summary.
    Requires a valid article name
    '''

    resp = rq.get(
        url = WIKI_API_URL,
        params = {
            "action": "query",
            "prop": "extracts",
            "exintro": "",
            "exsectionformat": "plain",
            "explaintext": "",
            "format": "json",
            "titles": article_name
        }
    )

    resp_dict = json.loads(resp.text)

    # print(json.dumps(resp_dict, indent=2))

    ## the extract key is nested 3 levels down
    level_2_down:dict = resp_dict["query"]["pages"]

    return level_2_down[list(level_2_down.keys())[0]]["extract"]

    # return json.loads(resp.text)


def summary(query: str):
    return get_article_summary(get_search_result(query))

def summary_random(query: str):
    return get_article_summary(get_search_result(query, random.randint(1,RANDOM_LIMIT)))

# x = get_search_result("sway")
# print(x)
# print(get_article_summary(x))
