#!/bin/env python3
# Run build scripts within a Docker container
import argparse
import subprocess
import os

DOCKER = "docker"
IMAGE_NAME = "warrior4-vm-ubuntu-build-env"
CONTAINER_NAME = "warrior4-vm-build-env"


def main():
    parser = argparse.ArgumentParser()

    parser.add_argument("--host", help="Docker daemon address")

    subparsers = parser.add_subparsers(required=True)

    init_parser = subparsers.add_parser(
        "init", help="Create Docker build environment image"
    )
    init_parser.set_defaults(func=init)

    remove_parser = subparsers.add_parser(
        "remove", help="Delete Docker build environment image and any build container"
    )
    remove_parser.set_defaults(func=remove)

    build_parser = subparsers.add_parser(
        "build", help="run build.sh within a build container"
    )
    build_parser.set_defaults(func=build)
    group = build_parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--device", help="Host device to container (a /dev/nbd* device)")
    group.add_argument(
        "--privileged",
        help="Allow container to access root devices (very risky!)",
        action="store_true",
    )

    apk_parser = subparsers.add_parser(
        "apk", help="run apk.sh within a build container"
    )
    apk_parser.set_defaults(func=apk)

    args = parser.parse_args()
    args.func(args)


def init(args):
    print(f"Creating image {IMAGE_NAME}")
    subprocess.run(
        common_docker_args(args) + ["build", "--file", "script/Dockerfile", "--tag", IMAGE_NAME, "."], check=True
    )


def remove(args):
    remove_container()

    print("Removing image")
    subprocess.run(common_docker_args(args) + ["image", "rm", IMAGE_NAME], check=True)


def build(args):
    set_up_cargo_cache()

    device_args = []

    if args.device:
        device_args.extend(
            [
                "--device",
                args.device,
            ]
        )
    elif args.privileged:
        device_args.append("--privileged")

    try:
        print("Running build.sh in container")
        proc_args = (
            common_docker_args(args)
            + [
                "container",
                "run",
                "--name",
                CONTAINER_NAME,
            ]
            + common_container_args()
            + device_args
            + [
                IMAGE_NAME,
                "./script/build.sh",
            ]
        )
        subprocess.run(
            proc_args,
            check=True,
        )
    finally:
        remove_container()


def apk(args):
    try:
        set_up_cargo_cache()

        print("Running apk.sh in container")
        proc_args = (
            common_docker_args(args)
            + ["container", "run", "--name", CONTAINER_NAME]
            + common_container_args()
            + [
                IMAGE_NAME,
                "./script/apk.sh",
            ]
        )
        subprocess.run(
            proc_args,
            check=True,
        )
    finally:
        remove_container()


def set_up_cargo_cache():
    os.makedirs("./target/registry/", exist_ok=True)


def common_docker_args(args):
    prog_args = [DOCKER]

    if args.host:
        prog_args.append("--host")
        prog_args.append(args.host)

    return prog_args


def common_container_args():
    return [
        "--mount",
        "type=bind,source=./,target=/home/ubuntu/warrior4-vm/",
        "--mount",
        "type=bind,source=./target/registry/,target=/home/ubuntu/.cargo/registry/",
        "--workdir",
        "/home/ubuntu/warrior4-vm/",
    ]


def remove_container():
    result = subprocess.run(
        [DOCKER, "container", "inspect", CONTAINER_NAME], stdout=subprocess.DEVNULL
    )

    if result.returncode == 0:
        print(f"Removing container {CONTAINER_NAME}")
        subprocess.run(
            [DOCKER, "container", "rm", "--force", CONTAINER_NAME], check=True
        )


if __name__ == "__main__":
    main()
