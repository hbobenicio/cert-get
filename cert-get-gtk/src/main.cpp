/**
 * clang++ -Wall -std=c++17 `pkg-config --cflags gtk+-3.0` -o cert-get-gtk src/main.cpp `pkg-config --libs gtk+-3.0` -L ../target/debug/ -lcert_get_core_ffi
 * LD_LIBRARY_PATH=../target/debug ./cert-get-gtk
 */
#include <gtk/gtk.h>

extern "C" {
    void say_hello();
}

static void activate(GtkApplication* app, gpointer user_data) {
//    GtkWidget *window = gtk_application_window_new(app);
//    gtk_window_set_title(GTK_WINDOW(window), "Window");
//    gtk_window_set_default_size(GTK_WINDOW(window), 200, 200);
//    gtk_widget_show_all(window);
    say_hello();

    GtkBuilder* builder = gtk_builder_new_from_file("main_dialog.glade");
    GObject* main_dialog = gtk_builder_get_object(builder, "main_dialog");
    g_signal_connect(main_dialog, "destroy", G_CALLBACK(gtk_main_quit), nullptr);

    gtk_application_add_window(app, GTK_WINDOW(main_dialog));
    gtk_widget_show_all(GTK_WIDGET(main_dialog));

    g_object_unref(builder);
}

int main(int argc, char **argv) {
    GtkApplication *app = gtk_application_new("org.gtk.example", G_APPLICATION_FLAGS_NONE);
    g_signal_connect(app, "activate", G_CALLBACK(activate), NULL);

    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);

    return status;
}
