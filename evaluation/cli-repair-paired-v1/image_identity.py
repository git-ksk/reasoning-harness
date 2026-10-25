"""Derive admissible image IDs from the hash-bound archive's actual bytes."""
import hashlib,json,tarfile

def verified_archive_ids(path,expected_sha256):
    with path.open('rb') as stream:
        assert hashlib.file_digest(stream,'sha256').hexdigest()==expected_sha256,'image_archive_changed'
    with tarfile.open(path,'r:gz') as archive:
        manifest=json.load(archive.extractfile('manifest.json'))
        assert len(manifest)==1
        config=archive.extractfile(manifest[0]['Config']).read()
        config_id='sha256:'+hashlib.sha256(config).hexdigest()
        ids={config_id}
        if 'index.json' in archive.getnames():
            index=json.load(archive.extractfile('index.json'))
            assert len(index['manifests'])==1
            digest=index['manifests'][0]['digest']
            assert digest.startswith('sha256:') and len(digest)==71
            blob=archive.extractfile('blobs/sha256/'+digest[7:]).read()
            assert 'sha256:'+hashlib.sha256(blob).hexdigest()==digest
            assert json.loads(blob)['config']['digest']==config_id
            ids.add(digest)
    return ids

def verify_loaded_image(path,expected_sha256,reviewed_id,actual_id):
    ids=verified_archive_ids(path,expected_sha256)
    assert reviewed_id in ids and actual_id in ids,'loaded_image_not_in_reviewed_archive'
    return ids
