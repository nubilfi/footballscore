version = $(shell awk '/^version/' Cargo.toml | head -n1 | cut -d "=" -f 2 | sed 's: ::g')
release := "1"
uniq := $(shell head -c1000 /dev/urandom | sha512sum | head -c 12 ; echo ;)
cidfile := "/tmp/.tmp.docker.$(uniq)"
build_type := release

all:
	mkdir -p build/ && \
	cp scripts/Dockerfile build/Dockerfile && \
	cp -a Cargo.toml src scripts Makefile build/ && \
	cd build/ && \
	docker build -t footballscore/build_rust:latest . && \
	cd ../ && \
	rm -rf build/

cleanup:
	docker rmi `docker images | python -c "import sys; print('\n'.join(l.split()[2] for l in sys.stdin if '<none>' in l))"`
	rm -rf $(cidfile) Dockerfile

pull:
	# `aws ecr get-login-password --region ap-southeast-1` <= AWS Cli v2
	docker pull public.ecr.aws/docker/library/rust:alpine3.18
	docker tag public.ecr.aws/docker/library/rust:alpine3.18 rust_stable:latest
	docker rmi public.ecr.aws/docker/library/rust:alpine3.18

get_version:
	echo $(version)
