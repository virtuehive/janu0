pipeline {
  agent { label 'MacMini' }
  options { skipDefaultCheckout() }
  parameters {
    gitParameter(name: 'GIT_TAG',
                 type: 'PT_BRANCH_TAG',
                 description: 'The Git tag to checkout. If not specified "master" will be checkout.',
                 defaultValue: 'master')
    string(name: 'RUST_TOOLCHAIN',
           description: 'The version of rust toolchain to use (e.g. stable)',
           defaultValue: 'stable')
    string(name: 'DOCKER_TAG',
           description: 'An extra Docker tag (e.g. "latest"). By default GIT_TAG will also be used as Docker tag',
           defaultValue: '')
    booleanParam(name: 'BUILD_MACOSX',
                 description: 'Build macosx target.',
                 defaultValue: true)
    booleanParam(name: 'BUILD_DOCKER',
                 description: 'Build Docker image.',
                 defaultValue: true)
    booleanParam(name: 'BUILD_LINUX64',
                 description: 'Build x86_64-unknown-linux-gnu target.',
                 defaultValue: true)
    booleanParam(name: 'BUILD_LINUX32',
                 description: 'Build i686-unknown-linux-gnu target.',
                 defaultValue: true)
    booleanParam(name: 'BUILD_AARCH64',
                 description: 'Build aarch64-unknown-linux-gnu target.',
                 defaultValue: true)
    booleanParam(name: 'BUILD_WIN64',
                 description: 'Build x86_64-pc-windows-gnu target.',
                 defaultValue: true)
    booleanParam(name: 'BUILD_WIN32',
                 description: 'Build i686-pc-windows-gnu target.',
                 defaultValue: true)
    booleanParam(name: 'PUBLISH_ECLIPSE_DOWNLOAD',
                 description: 'Publish the resulting artifacts to Eclipse download.',
                 defaultValue: false)
    booleanParam(name: 'PUBLISH_CRATES_IO',
                 description: 'Publish the resulting artifacts to crates.io.',
                 defaultValue: false)
    booleanParam(name: 'PUBLISH_DOCKER_HUB',
                 description: 'Publish the resulting artifacts to DockerHub.',
                 defaultValue: false)
  }
  environment {
      LABEL = get_label()
      DOWNLOAD_DIR="/home/data/httpd/download.eclipse.org/janu/janu/${LABEL}"
      MACOSX_DEPLOYMENT_TARGET=10.7
  }

  stages {
    stage('[MacMini] Checkout Git TAG') {
      steps {
        deleteDir()
        checkout([$class: 'GitSCM',
                  branches: [[name: "${params.GIT_TAG}"]],
                  doGenerateSubmoduleConfigurations: false,
                  extensions: [],
                  gitTool: 'Default',
                  submoduleCfg: [],
                  userRemoteConfigs: [[url: 'https://github.com/eclipse-janu/janu.git']]
                ])
      }
    }
    stage('[MacMini] Update Rust env') {
      steps {
        sh '''
        env
        rustup default ${RUST_TOOLCHAIN}
        '''
      }
    }

    stage('[MacMini] Build and tests') {
      when { expression { return params.BUILD_MACOSX }}
      steps {
        sh '''
        echo "Building eclipse-janu-${LABEL}"
        cargo build --release --all-targets
        cargo test --release
        '''
      }
    }

    stage('[MacMini] MacOS Package') {
      when { expression { return params.BUILD_MACOSX }}
      steps {
        sh '''
        tar -czvf eclipse-janu-${LABEL}-macosx${MACOSX_DEPLOYMENT_TARGET}-x86-64.tgz --strip-components 2 target/release/janud target/release/*.dylib
        tar -czvf eclipse-janu-${LABEL}-examples-macosx${MACOSX_DEPLOYMENT_TARGET}-x86-64.tgz --exclude 'target/release/examples/*.*' --strip-components 3 target/release/examples/*
        '''
      }
    }

    stage('[MacMini] Docker build') {
      when { expression { return params.BUILD_DOCKER }}
      steps {
        sh '''
        docker run --init --rm -v $(pwd):/workdir -w /workdir adlinktech/janu-dev-x86_64-unknown-linux-musl \
          /bin/ash -c "\
            rustup default ${RUST_TOOLCHAIN} && \
            cargo build --release --bins --lib --examples \
          "
        if [ -n "${DOCKER_TAG}" ]; then
          export EXTRA_TAG="-t eclipse/janu:${DOCKER_TAG}"
        fi
        docker build -t eclipse/janu:${LABEL} ${EXTRA_TAG} .
        '''
      }
    }

    stage('[MacMini] x86_64-unknown-linux-musl Package') {
      when { expression { return params.BUILD_DOCKER }}
      steps {
        sh '''
        tar -czvf eclipse-janu-${LABEL}-x86_64-unknown-linux-musl.tgz --strip-components 3 target/x86_64-unknown-linux-musl/release/janud target/x86_64-unknown-linux-musl/release/*.so
        tar -czvf eclipse-janu-${LABEL}-examples-x86_64-unknown-linux-musl.tgz --exclude 'target/x86_64-unknown-linux-musl/release/examples/*.*' --exclude 'target/x86_64-unknown-linux-musl/release/examples/*-*' --strip-components 4 target/x86_64-unknown-linux-musl/release/examples/*
        '''
      }
    }

    stage('[MacMini] x86_64-unknown-linux-gnu build') {
      when { expression { return params.BUILD_LINUX64 }}
      steps {
        sh '''
        docker run --init --rm -v $(pwd):/workdir -w /workdir adlinktech/janu-dev-manylinux2010-x86_64-gnu \
          /bin/bash -c "\
            rustup default ${RUST_TOOLCHAIN} && \
            cargo build --release --bins --lib --examples && \
            if [[ ${GIT_TAG} != origin/* ]]; then \
              cargo deb -p janu && \
              cargo deb -p janu-plugin-rest && \
              cargo deb -p janu-plugin-storages && \
              ./gen_janu_deb.sh x86_64-unknown-linux-gnu amd64 \
            ;fi \
          "
        '''
      }
    }
    stage('[MacMini] x86_64-unknown-linux-gnu Package') {
      when { expression { return params.BUILD_LINUX64 }}
      steps {
        sh '''
        tar -czvf eclipse-janu-${LABEL}-x86_64-unknown-linux-gnu.tgz --strip-components 3 target/x86_64-unknown-linux-gnu/release/janud target/x86_64-unknown-linux-gnu/release/*.so
        tar -czvf eclipse-janu-${LABEL}-examples-x86_64-unknown-linux-gnu.tgz --exclude 'target/x86_64-unknown-linux-gnu/release/examples/*.*' --exclude 'target/x86_64-unknown-linux-gnu/release/examples/*-*' --strip-components 4 target/x86_64-unknown-linux-gnu/release/examples/*
        '''
      }
    }

    stage('[MacMini] i686-unknown-linux-gnu build') {
      when { expression { return params.BUILD_LINUX32 }}
      steps {
        sh '''
        docker run --init --rm -v $(pwd):/workdir -w /workdir adlinktech/janu-dev-manylinux2010-i686-gnu \
          /bin/bash -c "\
            rustup default ${RUST_TOOLCHAIN} && \
            cargo build --release --bins --lib --examples && \
            if [[ ${GIT_TAG} != origin/* ]]; then \
              cargo deb -p janu && \
              cargo deb -p janu-plugin-rest && \
              cargo deb -p janu-plugin-storages && \
              ./gen_janu_deb.sh i686-unknown-linux-gnu i386 \
            ;fi \
          "
        '''
      }
    }
    stage('[MacMini] i686-unknown-linux-gnu Package') {
      when { expression { return params.BUILD_LINUX32 }}
      steps {
        sh '''
        tar -czvf eclipse-janu-${LABEL}-i686-unknown-linux-gnu.tgz --strip-components 3 target/i686-unknown-linux-gnu/release/janud target/i686-unknown-linux-gnu/release/*.so
        tar -czvf eclipse-janu-${LABEL}-examples-i686-unknown-linux-gnu.tgz --exclude 'target/i686-unknown-linux-gnu/release/examples/*.*' --exclude 'target/i686-unknown-linux-gnu/release/examples/*-*' --strip-components 4 target/x86_64-unknown-linux-gnu/release/examples/*
        '''
      }
    }

    stage('[MacMini] aarch64-unknown-linux-gnu build') {
      when { expression { return params.BUILD_AARCH64 }}
      steps {
        sh '''
        docker run --init --rm -v $(pwd):/workdir -w /workdir adlinktech/janu-dev-manylinux2014-aarch64-gnu \
          /bin/bash -c "\
            rustup default ${RUST_TOOLCHAIN} && \
            cargo build --release --bins --lib --examples && \
            if [[ ${GIT_TAG} != origin/* ]]; then
              cargo deb -p janu && \
              cargo deb -p janu-plugin-rest && \
              cargo deb -p janu-plugin-storages && \
              ./gen_janu_deb.sh aarch64-unknown-linux-gnu aarch64 \
            ;fi \
          "
        '''
      }
    }
    stage('[MacMini] aarch64-unknown-linux-gnu Package') {
      when { expression { return params.BUILD_AARCH64 }}
      steps {
        sh '''
        tar -czvf eclipse-janu-${LABEL}-aarch64-unknown-linux-gnu.tgz --strip-components 3 target/aarch64-unknown-linux-gnu/release/janud target/aarch64-unknown-linux-gnu/release/*.so
        tar -czvf eclipse-janu-${LABEL}-aarch64-unknown-linux-gnu.tgz --exclude 'target/aarch64-unknown-linux-gnu/release/examples/*.*' --exclude 'target/aarch64-unknown-linux-gnu/release/examples/*-*' --strip-components 4 target/aarch64-unknown-linux-gnu/release/examples/*
        '''
      }
    }

    stage('[MacMini] x86_64-pc-windows-gnu build') {
      when { expression { return params.BUILD_WIN64 }}
      steps {
        sh '''
        cargo build --release --bins --lib --examples --target=x86_64-pc-windows-gnu
        '''
      }
    }
    stage('[MacMini] x86_64-pc-windows-gnu Package') {
      when { expression { return params.BUILD_WIN64 }}
      steps {
        sh '''
        zip eclipse-janu-${LABEL}-x86_64-pc-windows-gnu.zip --junk-paths target/x86_64-pc-windows-gnu/release/janud.exe target/x86_64-pc-windows-gnu/release/*.dll
        zip eclipse-janu-${LABEL}-examples-x86_64-pc-windows-gnu.zip --exclude 'target/x86_64-pc-windows-gnu/release/examples/*-*' --junk-paths target/x86_64-pc-windows-gnu/release/examples/*.exe
        '''
      }
    }

    stage('[MacMini] i686-pc-windows-gnu build') {
      when { expression { return params.BUILD_WIN32 }}
      steps {
        sh '''
        cargo build --release --bins --lib --examples --target=i686-pc-windows-gnu
        '''
      }
    }
    stage('[MacMini] i686-pc-windows-gnu Package') {
      when { expression { return params.BUILD_WIN32 }}
      steps {
        sh '''
        zip eclipse-janu-${LABEL}-i686-pc-windows-gnu.zip --junk-paths target/i686-pc-windows-gnu/release/janud.exe target/i686-pc-windows-gnu/release/*.dll
        zip eclipse-janu-${LABEL}-examples-i686-pc-windows-gnu.zip --exclude 'target/i686-pc-windows-gnu/release/examples/*-*' --junk-paths target/i686-pc-windows-gnu/release/examples/*.exe
        '''
      }
    }

    stage('[MacMini] Prepare directory on download.eclipse.org') {
      when { expression { return params.PUBLISH_ECLIPSE_DOWNLOAD }}
      steps {
        // Note: remove existing dir on download.eclipse.org only if it's for a branch
        // (e.g. master that is rebuilt periodically from different commits)
        sshagent ( ['projects-storage.eclipse.org-bot-ssh']) {
          sh '''
            if [[ ${GIT_TAG} == origin/* ]]; then
              ssh genie.janu@projects-storage.eclipse.org rm -fr ${DOWNLOAD_DIR}
            fi
            ssh genie.janu@projects-storage.eclipse.org mkdir -p ${DOWNLOAD_DIR}
            COMMIT_ID=`git log -n1 --format="%h"`
            echo "https://github.com/eclipse-janu/janu/tree/${COMMIT_ID}" > _git_commit_${COMMIT_ID}.txt
            rustc --version > _rust_toolchain_${RUST_TOOLCHAIN}.txt
            scp _*.txt genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/
          '''
        }
      }
    }

    stage('[MacMini] Publish janu-macosx to download.eclipse.org') {
      when { expression { return params.PUBLISH_ECLIPSE_DOWNLOAD && params.BUILD_MACOSX }}
      steps {
        sshagent ( ['projects-storage.eclipse.org-bot-ssh']) {
          sh '''
            ssh genie.janu@projects-storage.eclipse.org mkdir -p ${DOWNLOAD_DIR}
            scp eclipse-janu-${LABEL}-*macosx*.tgz genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/
          '''
        }
      }
    }

    stage('[MacMini] Publish janu-x86_64-unknown-linux-musl to download.eclipse.org') {
      when { expression { return params.PUBLISH_ECLIPSE_DOWNLOAD && params.BUILD_DOCKER }}
      steps {
        sshagent ( ['projects-storage.eclipse.org-bot-ssh']) {
          sh '''
            ssh genie.janu@projects-storage.eclipse.org mkdir -p ${DOWNLOAD_DIR}
            scp eclipse-janu-${LABEL}-*x86_64-unknown-linux-musl.tgz genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/
          '''
        }
      }
    }

    stage('[MacMini] Publish janu-x86_64-unknown-linux-gnu to download.eclipse.org') {
      when { expression { return params.PUBLISH_ECLIPSE_DOWNLOAD && params.BUILD_LINUX64 }}
      steps {
        sshagent ( ['projects-storage.eclipse.org-bot-ssh']) {
          sh '''
            ssh genie.janu@projects-storage.eclipse.org mkdir -p ${DOWNLOAD_DIR}
            scp eclipse-janu-${LABEL}-*x86_64-unknown-linux-gnu.tgz genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/
            if [[ ${GIT_TAG} != origin/* ]]; then
              scp target/x86_64-unknown-linux-gnu/debian/*.deb genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/
            fi
          '''
        }
      }
    }

    stage('[MacMini] Publish janu-i686-unknown-linux-gnu to download.eclipse.org') {
      when { expression { return params.PUBLISH_ECLIPSE_DOWNLOAD && params.BUILD_LINUX32 }}
      steps {
        sshagent ( ['projects-storage.eclipse.org-bot-ssh']) {
          sh '''
            ssh genie.janu@projects-storage.eclipse.org mkdir -p ${DOWNLOAD_DIR}
            scp eclipse-janu-${LABEL}-*i686-unknown-linux-gnu.tgz genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/
            if [[ ${GIT_TAG} != origin/* ]]; then
              scp target/i686-unknown-linux-gnu/debian/*.deb genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/
            fi
          '''
        }
      }
    }

    stage('[MacMini] Publish janu-x86_64-pc-windows-gnu to download.eclipse.org') {
      when { expression { return params.PUBLISH_ECLIPSE_DOWNLOAD && params.BUILD_WIN64 }}
      steps {
        sshagent ( ['projects-storage.eclipse.org-bot-ssh']) {
          sh '''
            ssh genie.janu@projects-storage.eclipse.org mkdir -p ${DOWNLOAD_DIR}
            scp eclipse-janu-${LABEL}-*x86_64-pc-windows-gnu.zip genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/
          '''
        }
      }
    }

    stage('[MacMini] Publish janu-i686-pc-windows-gnu to download.eclipse.org') {
      when { expression { return params.PUBLISH_ECLIPSE_DOWNLOAD && params.BUILD_WIN32 }}
      steps {
        sshagent ( ['projects-storage.eclipse.org-bot-ssh']) {
          sh '''
            ssh genie.janu@projects-storage.eclipse.org mkdir -p ${DOWNLOAD_DIR}
            scp eclipse-janu-${LABEL}-*i686-pc-windows-gnu.zip genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/
          '''
        }
      }
    }

    stage('[UbuntuVM] Build Packages.gz for download.eclipse.org') {
      agent { label 'UbuntuVM' }
      when { expression { return params.PUBLISH_ECLIPSE_DOWNLOAD && !env.GIT_TAG.startsWith('origin/') && (params.BUILD_LINUX64 || params.BUILD_LINUX32) }}
      steps {
        deleteDir()
        sshagent ( ['projects-storage.eclipse.org-bot-ssh']) {
          sh '''
          scp genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/*.deb ./
          dpkg-scanpackages --multiversion . > Packages
          cat Packages
          gzip -c9 < Packages > Packages.gz
          scp Packages.gz genie.janu@projects-storage.eclipse.org:${DOWNLOAD_DIR}/
          '''
        }
      }
    }

    stage('[MacMini] Publish to crates.io') {
      when { expression { return params.PUBLISH_CRATES_IO }}
      steps {
        sh '''
        cd janu-util && cargo publish && cd - && sleep 30
        cd janu && cargo publish && cd - && sleep 30
        cd plugins/janu-plugin-trait && cargo publish && cd - && sleep 30
        cd backends/traits && cargo publish && cd -
        cd plugins/janu-plugin-rest && cargo publish && cd -
        '''
      }
    }

    stage('[MacMini] Publish to Docker Hub') {
      when { expression { return params.PUBLISH_DOCKER_HUB && params.BUILD_DOCKER}}
      steps {
        withCredentials([usernamePassword(credentialsId: 'dockerhub-bot',
            passwordVariable: 'DOCKER_HUB_CREDS_PSW', usernameVariable: 'DOCKER_HUB_CREDS_USR')])
        {
          sh '''
          docker login -u ${DOCKER_HUB_CREDS_USR} -p ${DOCKER_HUB_CREDS_PSW}
          docker push eclipse/janu:${LABEL}
          if [ -n "${DOCKER_TAG}" ]; then
            docker push eclipse/janu:${DOCKER_TAG}
          fi
          docker logout
          '''
        }
      }
    }
  }
}

def get_label() {
    return env.GIT_TAG.startsWith('origin/') ? env.GIT_TAG.minus('origin/') : env.GIT_TAG
}
