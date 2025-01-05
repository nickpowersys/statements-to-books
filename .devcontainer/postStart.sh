git config --global --add safe.directory ${containerWorkspaceFolder}
echo "/root/.local/share/uv/python/cpython-3.11.11-linux-aarch64-gnu/lib" | sudo tee /etc/ld.so.conf.d/uv_python.conf
sudo ldconfig
