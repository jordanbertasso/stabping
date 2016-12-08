#!/usr/bin/env python3

import sys
import os


def print_e(*pos, **kwargs):
    print(*pos, file=sys.stderr, **kwargs)


class BuildError(Exception):
    def __init__(self, message):
        self.message = message


class Environment:
    is_host = False  # whether this is a native build
    target = None  # the target triplet
    os_type = None  # one of 'linux', 'osx', 'win'
    root_dir = None  # the project directory for the build
    release_version = None
    release_build = False
    github_repo_owner = 'icasdri'
    github_repo_name = 'stabping'
    github_release_api_token = None

    def init_environment(self):
        if 'TRAVIS' in os.environ:
            self.is_host = os.getenv('IS_HOST', False)
            self.target = os.getenv('TARGET', None)
            if not self.target:
                raise BuildError('TARGET not specified')

            self.os_type = os.getenv('TRAVIS_OS_NAME', 'linux')
            self.root_dir = os.getenv('TRAVIS_BUILD_DIR', self.root_dir)

            tag = os.getenv('TRAVIS_TAG', None)
            self.release_build = tag and len(tag.strip()) > 0
            self.release_version = tag

            self.can_release = os.getenv('CAN_RELEASE', False)
            self.github_release_api_token = os.getenv('SEC_GH_API_KEY')
        elif 'APPVEYOR' in os.environ:
            self.is_host = True
            self.target = os.getenv('TARGET', None)
            if not self.target:
                raise BuildError('TARGET not specified')

            self.os_type = 'windows'
            self.root_dir = os.getenv('APPVEYOR_BUILD_FOLDER', self.root_dir)
            tag = os.getenv('APPVEYOR_REPO_TAG_NAME', None)
            self.release_build = tag and len(tag.strip()) > 0
            self.release_version = tag

            self.can_release = os.getenv('CAN_RELEASE', False)
            self.github_release_api_token = os.getenv('SEC_GH_API_KEY')

    def cd_root(self):
        os.chdir(self.root_dir)

    def path(self, target):
        return os.path.join(self.root_dir, target)


ENV = Environment()


def consolidate_artifacts():
    ENV.cd_root()
    print_e('------ Artifact Consolidation Routine ------')

    if ENV.os_type == 'windows':
        binary = ENV.path(r'target\{}\release\stabping.exe'.format(ENV.target))
    else:
        binary = ENV.path('target/{}/release/stabping'.format(ENV.target))
    target_zip = 'stabping-{}-{}.zip'.format(ENV.release_version, ENV.target)

    sample_cfg = ENV.path('stabping_config.json')

    if os.path.isfile(binary):
        from zipfile import ZipFile
        print_e('Zipping binary to: ' + target_zip)
        with ZipFile(target_zip, 'w') as zipped:
            zipped.write(binary, arcname=os.path.basename(binary))
            zipped.write(sample_cfg, arcname=os.path.basename(sample_cfg))
            zipped.write(ENV.path('README.md'), arcname='README.md')
            zipped.write(ENV.path('COPYING'), arcname='COPYING')
            zipped.write(ENV.path('LICENSE'), arcname='LICENSE')
    else:
        raise BuildError('Failed to find binary: {}'.format(binary))

    return target_zip


def deploy_release(target_zip):
    ENV.cd_root()
    print_e('------ Release Deployment Routine ------')

    from urllib.request import urlopen, Request
    from urllib.error import HTTPError, URLError
    import json

    def repo_api(additional):
        return 'https://api.github.com/repos/{}/{}/{}?access_token={}'.format(
                ENV.github_repo_owner, ENV.github_repo_name, additional,
                ENV.github_release_api_token)

    release_obj = None
    try:
        print_e('Checking for existing GitHub Release in latest...')
        latest_response = urlopen(repo_api('releases/latest'))

        latest_obj = json.loads(latest_response.read())
        if latest_obj.get('tag_name', None) == ENV.release_version:
            print_e('Found existing GitHub Release.')
            release_obj = latest_obj
    except HTTPError as e:
        if e.code != 404:
            raise BuildError('Unexpected HTTP response from GitHub API: '
                             '{} {}'.format(e.getcode(), e.msg))

    if release_obj is None:
        print_e('Checking for existing GitHub Release across all releases...')
        all_releases_response = urlopen(repo_api('releases'))
        all_releases = json.loads(all_releases_response.read().decode())
        for release_i in all_releases:
            if release_i.get('tag_name', None) == ENV.release_version:
                print_e('Found existing GitHub Release.')
                release_obj = release_i
                break

    if release_obj is None:
        print_e('Existing GitHub Release not found. Creating new one...')
        new_release_data = json.dumps({
            'tag_name': ENV.release_version,
            'name': 'Pending Release',
            'body': 'Please wait while release builds finish and artifacts'
                    ' are uploaded. This release will be available soon.',
            'draft': True
        }).encode()
        request = Request(repo_api('releases'), new_release_data,
                          {'Content-Type': 'application/json'})

        response = urlopen(request)
        release_obj = json.loads(response.read().decode())

    upload_url = None
    if 'upload_url' in release_obj:
        print_e('Building asset upload url...')
        upload_url = release_obj['upload_url']
        upload_url = upload_url[:upload_url.rfind('{')]  # chop off template
        upload_url = upload_url + '?name={}&access_token={}'.format(
                        target_zip, ENV.github_release_api_token)
    else:
        raise BuildError('Unexpected JSON response from GitHub API: '
                         'no upload_url in returned object!')

    with open(target_zip, 'rb') as target_zip_raw:
        print_e('Reading data for file to upload...')
        target_zip_data = target_zip_raw.read()

    request = Request(upload_url, target_zip_data,
                      {'Content-Type': 'application/zip'})
    try:
        print_e('Uploading release asset...')
        response = urlopen(request)
    except URLError as e:
        if 'Broken pipe' in e.reason:
            raise BuildError('Failed to upload release asset. Either a '
                             'connection problem was encountered or GitHub '
                             'API rejected it (possibly duplicate filename?).')

    if response.getcode() == 201:  # HTTP code for 'Created'
        print_e('Release asset upload successful.')
    else:
        raise BuildError('Failed to upload release asset. GitHub API '
                         'responded: {} {}'.format(response.get_code,
                                                   response.reason))


ENV.init_environment()

if ENV.release_build:
    try:
        target_zip = consolidate_artifacts()
        deploy_release(target_zip)
    except BuildError as e:
        print_e(e.message)
        raise e
else:
    print_e("Not a release build, skipping release.")
