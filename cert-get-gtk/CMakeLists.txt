cmake_minimum_required(VERSION 3.18)
  
project(cert-get-gtk
    VERSION 0.1.0
    DESCRIPTION "GTK Gui for cert-get - download certificates from HTTPS websites"
    LANGUAGES CXX
)

# include("${CMAKE_BINARY_DIR}/conanbuildinfo.cmake")
# conan_basic_setup(TARGETS)

# Use the package PkgConfig to detect GTK+ headers/library files
FIND_PACKAGE(PkgConfig REQUIRED)
PKG_CHECK_MODULES(GTK3 REQUIRED "gtk+-3.0")

set(CPP_SOURCES
    "${PROJECT_SOURCE_DIR}/src/main.cpp"
)

add_executable(${PROJECT_NAME} "${CPP_SOURCES}")
target_compile_features(${PROJECT_NAME} PRIVATE cxx_std_17)
target_compile_options(${PROJECT_NAME} PRIVATE "-Wall")
target_include_directories(${PROJECT_NAME}
    PRIVATE
        ${GTK3_INCLUDE_DIRS}
        # CONAN_PKG::nlohmann_json
)
target_link_libraries(${PROJECT_NAME}
    PRIVATE
        ${pkgcfg_lib_GTK3_gtk-3}
        ${pkgcfg_lib_GTK3_gdk-3}
        ${pkgcfg_lib_GTK3_pangocairo-1.0}
        ${pkgcfg_lib_GTK3_pango-1.0}
        ${pkgcfg_lib_GTK3_atk-1.0}
        ${pkgcfg_lib_GTK3_cairo-gobject}
        ${pkgcfg_lib_GTK3_cairo}
        ${pkgcfg_lib_GTK3_gdk_pixbuf-2.0}
        ${pkgcfg_lib_GTK3_gio-2.0}
        ${pkgcfg_lib_GTK3_gobject-2.0}
        ${pkgcfg_lib_GTK3_glib-2.0}
        "${PROJECT_SOURCE_DIR}/../target/debug/libcert_get_core_ffi.so"
        # CONAN_PKG::nlohmann_json
)

add_custom_command(
    TARGET ${PROJECT_NAME} POST_BUILD
    COMMAND
        ${CMAKE_COMMAND} -E copy
            # "${CMAKE_SOURCE_DIR}/src/run.sh"
            "${CMAKE_SOURCE_DIR}/src/main_dialog.glade"
            $<TARGET_FILE_DIR:${PROJECT_NAME}>
    # DEPENDS ${destination}
    COMMENT "Copying runtime resources"
)
