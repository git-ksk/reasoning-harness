"""Hash-only regular-file/mode/link comparison; no source-content inspection."""
from pathlib import Path,PurePosixPath
import tarfile,hashlib,json,io,os
OUT=Path(__file__).resolve().parent
OLD=Path('/private/tmp/engine073-realworld-M0-runtime-39205043793/engine073-realworld-M0-runtime-39205043793/runtime-image.tar.gz')
NEW=Path('/private/tmp/engine073-realworld-M0-runtime-39206178872/runtime-image.tar.gz')
def save(name,data):
 with (OUT/name).open('x') as f:json.dump(data,f,indent=2);f.write('\n');f.flush();os.fsync(f.fileno())
def scan(path):
 fs={}
 with tarfile.open(path) as t:
  manifest=json.loads(t.extractfile('manifest.json').read())[0]
  for layer in manifest['Layers']:
   with tarfile.open(fileobj=io.BytesIO(t.extractfile(layer).read())) as inner:
    for member in inner:
     name=str(PurePosixPath(member.name.removeprefix('./')));base=PurePosixPath(name).name
     if base.startswith('.wh.'):
      prefix=str(PurePosixPath(name).parent)+'/'
      victim=prefix+base[4:]
      for key in list(fs):
       if (base=='.wh..wh..opq' and key.startswith(prefix)) or key==victim or key.startswith(victim+'/'):del fs[key]
      continue
     if member.isfile():fs[name]={'sha256':hashlib.sha256(inner.extractfile(member).read()).hexdigest(),'mode':member.mode,'kind':'file'}
     elif member.issym() or member.islnk():fs[name]={'link':member.linkname,'mode':member.mode,'kind':'symbolic' if member.issym() else 'hardlink'}
 return fs
old,new=scan(OLD),scan(NEW)
diff={p:{'old':old.get(p),'new':new.get(p)} for p in sorted(old.keys()|new.keys()) if old.get(p)!=new.get(p)}
base_changes={k:v for k,v in diff.items() if not k.startswith('repo/') and not k.startswith('eval/bin/')}
libs={k:v for k,v in diff.items() if k.startswith(('usr/lib/','usr/local/','lib/','lib64/','bin/','usr/bin/'))}
save('runtime-filesystem-hash-comparison.json',{'old_file_or_link_count':len(old),'new_file_or_link_count':len(new),'changed':diff,'base_changed':base_changes,'interpreter_library_executable_changes':libs,'no_source_contents_read':True})
print(json.dumps({'old_files':len(old),'new_files':len(new),'changed_files':list(diff),'base_changed':list(base_changes),'functional_interpreter_library_changes':list(libs)},indent=2))
