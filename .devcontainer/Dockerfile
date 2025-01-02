FROM mcr.microsoft.com/devcontainers/rust:bullseye
COPY --from=ghcr.io/astral-sh/uv:0.5.13 /uv /uvx /bin/
ARG USERNAME=vscode

WORKDIR /app

# Enable bytecode compilation
ENV UV_COMPILE_BYTECODE=1

# Copy from the cache instead of linking since it's a mounted volume
ENV UV_LINK_MODE=copy

COPY . /app

# Install the project's dependencies using the lockfile and settings
RUN --mount=type=cache,target=/root/.cache/uv \
    --mount=type=bind,source=uv.lock,target=uv.lock \
    --mount=type=bind,source=pyproject.toml,target=pyproject.toml \
    uv venv --python 3.11 && uv sync --group dev

ENV VENV_DIR=.venv
RUN chown -R $USERNAME $VENV_DIR && chmod -R 755 $VENV_DIR

ENV STATEMENTS_TO_BOOKS_PROJECT=.
RUN chown -R $USERNAME $STATEMENTS_TO_BOOKS_PROJECT

# Place executables in the environment at the front of the path
ENV PATH="/app/.venv/bin:$PATH"

SHELL ["/bin/bash", "-c", "source .venv/bin/activate"]

ENTRYPOINT []

USER $USERNAME
