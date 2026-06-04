from setuptools import setup


setup(
    name="codefire-mvp",
    version="0.2.0",
    description="CodeFire MVP CLI",
    long_description=open("README.md", encoding="utf-8").read(),
    long_description_content_type="text/markdown",
    author="CodeFire MVP",
    license="UNLICENSED",
    python_requires=">=3.10",
    scripts=["codefire"],
)
