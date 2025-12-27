cmake_minimum_required(VERSION 3.14)

set(PROJECT_NAME "p2p_node")
project(${PROJECT_NAME} LANGUAGES CXX)

include("../cargokit/cmake/cargokit.cmake")

# --- SOLUCIÓN PARA SYMLINKS DE WINDOWS ---
# No usamos rutas absolutas (REALPATH) porque Cargokit ya las gestiona mal con symlinks.
# Usamos la ruta relativa estándar desde la carpeta 'windows' del plugin hacia el nodo.
set(RUST_NODE_PATH "../../../../../../rust/node")

# Llamada a cargokit con la ruta relativa
apply_cargokit(${PROJECT_NAME} "${RUST_NODE_PATH}" p2p_node "")
# -----------------------------------------

set(p2p_node_bundled_libraries
  "${${PROJECT_NAME}_cargokit_lib}"
  PARENT_SCOPE
)
