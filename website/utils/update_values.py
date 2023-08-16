import sys
import json
import subprocess

from datetime import datetime
from hashlib import sha256

version = sys.argv[1]

with open('../website/src/static/menial_2-linux.bin', 'rb') as menial:
    menial_hash = sha256(menial.read()).hexdigest()
    menial_hash = menial_hash[:32] + "<wbr>" + menial_hash[32:]


with open('../website/src/static/menial_2.tar.gz', 'rb') as menial_tar_gz:
    menial_tar_gz_hash = sha256(menial_tar_gz.read()).hexdigest()
    menial_tar_gz_hash = menial_tar_gz_hash[:32] + "<wbr>" + menial_tar_gz_hash[32:]


with open('../website/hashvalues.json', 'w+') as f:
    timestamp = datetime.strftime(datetime.now(), "%Y-%m-%d %H:%M:%S")
    d = {
        "menial_hash": menial_hash,
        "menial_tar_gz_hash": menial_tar_gz_hash,
        "timestamp": timestamp,
        "version": version,
        "os_info": subprocess.check_output(['uname','-mrs']).decode().strip()
    }
    f.write(json.dumps(d))
