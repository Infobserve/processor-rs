# Use this script to publish the events under
# ./fixtures/events.json to redis://localhost:6379/@events

import json
import redis

r = redis.Redis(host='localhost', port=6379, db=0)

with open('fixtures/events.json', 'r') as handle:
  c = handle.read()
  js = json.loads(c)

for j in js:
  r.publish("events", json.dumps(j))

r.publish("cmd", "quit")