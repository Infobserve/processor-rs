# Use this script to publish the events under
# ./fixtures/events.json to redis://localhost:6379/@events

import sys
import json
import redis

r = redis.Redis(host='localhost', port=6379, db=0)

fname = sys.argv[1] if len(sys.argv) == 2 else 'fixtures/events.json'
with open(fname, 'r') as handle:
  c = handle.read()
  js = json.loads(c)

for j in js:
  r.rpush("events", json.dumps(j))
  print(f"Published {j}")

r.rpush("events", "QUIT")
