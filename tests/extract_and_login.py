#!/usr/bin/env python3
import json, urllib.parse, subprocess, shlex, sys, os

# This script supports two modes:
# 1) If --token is provided on the command line, POST that token directly to the homeserver.
# 2) Otherwise, read captures.json and extract the first token found in a fluffychat callback URL.

def parse_args(argv):
    args = {}
    it = iter(argv[1:])
    for a in it:
        if a.startswith('--token='):
            args['token'] = a.split('=',1)[1]
        elif a.startswith('--user_id='):
            args['user_id'] = a.split('=',1)[1]
        elif a.startswith('--homeserver='):
            args['homeserver'] = a.split('=',1)[1]
    return args

args = parse_args(sys.argv)
homeserver = args.get('homeserver', 'https://test.aaca.eu.org')

token = args.get('token')
user_id = args.get('user_id')

if not token:
    CAP='captures.json'
    if not os.path.exists(CAP):
        print('captures.json not found in cwd')
        sys.exit(1)
    with open(CAP) as f:
        c=json.load(f)
    loc=None
    for e in c.get('captures',[]):
        # check location header
        h=e.get('headers') or {}
        if isinstance(h, dict):
            loc_hdr=h.get('location')
            if loc_hdr and 'token=' in loc_hdr:
                loc=loc_hdr
                break
        u=e.get('url') or ''
        if 'token=' in u and 'fluffychat.im' in u:
            loc=u
            break
    if not loc:
        print('NO_TOKEN_IN_CAPTURES')
        sys.exit(2)
    print('FOUND:', loc)
    # parse
    p=urllib.parse.urlparse(loc)
    frag=p.fragment
    qs=''
    if frag and '?' in frag:
        qs=frag.split('?',1)[1]
    elif frag and 'token=' in frag:
        qs=frag
    else:
        qs=p.query
    params=urllib.parse.parse_qs(qs)
    print('PARSED_QS:', params)
    if 'token' not in params:
        print('NO_TOKEN_PARAM')
        sys.exit(3)
    token=params['token'][0]
    user_id=params.get('user_id',[None])[0]

print('TOKEN:', token)
print('USER_ID:', user_id)

# build json body
body={'type':'m.login.token','token':token,'initial_device_display_name':'extract_and_login','refresh_token':False}
body_json=json.dumps(body)
url=f"{homeserver.rstrip('/')}/_matrix/client/v3/login"
cmd=['curl','-i','-s','-X','POST','-H','Content-Type: application/json','-d',body_json,url]
print('\nRUN:', ' '.join(shlex.quote(x) for x in cmd))
res=subprocess.run(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
out=res.stdout.decode()
if not out:
    out=res.stderr.decode()
print('\nRESPONSE:')
print(out)
# exit with curl exit code
sys.exit(res.returncode)
