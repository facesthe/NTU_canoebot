FROM python:3-slim-bullseye

WORKDIR /NTU_canoebot

COPY . .

RUN pip3 install -r ./.scripts/requirements.txt

# last
CMD [ "python3", "canoebot.py" ]
