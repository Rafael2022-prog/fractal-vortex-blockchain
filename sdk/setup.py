#!/usr/bin/env python3
"""
Setup script for FVChain Python SDK
"""

from setuptools import setup, find_packages
import os

# Read the contents of README file
this_directory = os.path.abspath(os.path.dirname(__file__))
with open(os.path.join(this_directory, 'README.md'), encoding='utf-8') as f:
    long_description = f.read()

setup(
    name="fvchain-sdk",
    version="1.0.0",
    author="FVChain Development Team",
    author_email="dev@fvchain.xyz",
    description="Official Python SDK for Fractal Vortex Chain (FVChain) blockchain integration",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/fvchain/fvchain-sdk-python",
    project_urls={
        "Bug Tracker": "https://github.com/fvchain/fvchain-sdk-python/issues",
        "Documentation": "https://docs.fvchain.xyz/sdk/python",
        "Source Code": "https://github.com/fvchain/fvchain-sdk-python",
        "Homepage": "https://fvchain.xyz"
    },
    packages=find_packages(),
    py_modules=["fvchain_sdk"],
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Internet :: WWW/HTTP :: Dynamic Content",
        "Topic :: Office/Business :: Financial",
        "Topic :: Security :: Cryptography",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.8",
    install_requires=[
        "requests>=2.25.0",
        "typing-extensions>=4.0.0; python_version<'3.10'"
    ],
    extras_require={
        "dev": [
            "pytest>=6.0",
            "pytest-cov>=2.0",
            "black>=21.0",
            "flake8>=3.8",
            "mypy>=0.800",
            "sphinx>=4.0",
            "sphinx-rtd-theme>=1.0"
        ],
        "test": [
            "pytest>=6.0",
            "pytest-cov>=2.0",
            "responses>=0.18.0"
        ]
    },
    keywords=[
        "blockchain",
        "cryptocurrency",
        "fvchain",
        "fractal",
        "vortex",
        "chain",
        "sdk",
        "api",
        "client",
        "rpc",
        "mining",
        "wallet",
        "defi",
        "web3"
    ],
    include_package_data=True,
    zip_safe=False,
    entry_points={
        "console_scripts": [
            "fvchain-cli=fvchain_sdk:main",
        ],
    },
)