
import requests as rq
import json

STR = "https://api.urbandictionary.com/v0/define?term=flump"

UB_API_URL = "https://api.urbandictionary.com/v0/define"

def get_search_result(query: str, position: int = 1) -> dict:
    '''Returns the '''
    resp = rq.get(
        url=UB_API_URL,
        params = {
            "term": query
        }
    )

    resp_json: list = json.loads(resp.text)["list"]
    if position > len(resp_json):
        return resp_json[-1]
    else:
        return resp_json[position-1]

def get_article_summary(article: dict) -> str:
    '''Gets the article definition param from an article struct'''
    try:
        return article["definition"]
    except:
        return None

def summary(query: str):
    return get_article_summary(get_search_result(query))
