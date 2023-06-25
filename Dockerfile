FROM python:3.11-slim-bookworm

WORKDIR /NTU_canoebot

COPY . .

RUN pip3 install -r ./.scripts/requirements.txt

ENV TZ="Asia/Singapore"

# last
CMD [ "python3", "canoebot.py" ]
