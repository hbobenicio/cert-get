/**
 * clang++ -Wall -std=c++17 `pkg-config --cflags gtk+-3.0` -o cert-get-gtk src/main.cpp `pkg-config --libs gtk+-3.0` -L ../target/debug/ -lcert_get_core_ffi
 * LD_LIBRARY_PATH=../target/debug ./cert-get-gtk
 * 
 * cd to cert-get/target/debug
 * cmake ../../cert-get-gtk
 * make
 * ./cert-get-gtk
 */
#include <iostream>
#include <gtk/gtk.h>

// TODO move this to the ffi library and expose this as a header
extern "C" {
    void download_certs(const char* url, const char* output_dir);
}

// TODO create a main_dialog class to encapsulate this
static GtkDialog* main_dialog = nullptr;
static GtkEntry* host_ip_entry = nullptr;
static GtkEntry* port_entry = nullptr;
static GtkButton* ok_button = nullptr;
static GtkButton* cancel_button = nullptr;
static GtkFileChooserButton* output_dir_file_chooser_button = nullptr;
static gchar* output_dir = nullptr;

static void activate(GtkApplication* app, gpointer user_data);
static void on_ok_button_clicked(GtkButton* ok_button, gpointer user_data);
static void on_cancel_button_clicked(GtkButton* sender, gpointer user_data);
static void on_output_dir_file_chooser_button_file_set(GtkFileChooserButton* file_chooser_button, gpointer user_data);
static void quit();

int main(int argc, char **argv) {
    GtkApplication *app = gtk_application_new("br.com.hbobenicio.certget.gtk", G_APPLICATION_FLAGS_NONE);
    g_signal_connect(app, "activate", G_CALLBACK(activate), nullptr);

    int status = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);

    return status;
}

static void activate(GtkApplication* app, gpointer user_data) {
    GtkBuilder* builder = gtk_builder_new_from_file("main_dialog.glade");
    gtk_builder_connect_signals(builder, nullptr);
    
    main_dialog = GTK_DIALOG(gtk_builder_get_object(builder, "main_dialog"));
    host_ip_entry = GTK_ENTRY(gtk_builder_get_object(builder, "host_ip_entry"));
    port_entry = GTK_ENTRY(gtk_builder_get_object(builder, "port_entry"));
    ok_button = GTK_BUTTON(gtk_builder_get_object(builder, "ok_button"));
    cancel_button = GTK_BUTTON(gtk_builder_get_object(builder, "cancel_button"));
    output_dir_file_chooser_button = GTK_FILE_CHOOSER_BUTTON(gtk_builder_get_object(builder, "output_dir_file_chooser_button"));

    g_signal_connect(main_dialog, "destroy", G_CALLBACK(gtk_main_quit), nullptr);
    g_signal_connect(cancel_button, "clicked", G_CALLBACK(on_cancel_button_clicked), nullptr);
    g_signal_connect(ok_button, "clicked", G_CALLBACK(on_ok_button_clicked), nullptr);
    g_signal_connect(output_dir_file_chooser_button, "file-set", G_CALLBACK(on_output_dir_file_chooser_button_file_set), nullptr);

    // TODO get rid of this warning:
    //   "GtkDialog mapped without a transient parent. This is discouraged."
    // by creating a hidden parent main_window
    gtk_application_add_window(app, GTK_WINDOW(main_dialog));
    gtk_widget_show_all(GTK_WIDGET(main_dialog));

    g_object_unref(builder);
}

static void on_output_dir_file_chooser_button_file_set(GtkFileChooserButton* file_chooser_button, gpointer user_data) {
    output_dir = gtk_file_chooser_get_filename(GTK_FILE_CHOOSER(file_chooser_button));
}

static void on_ok_button_clicked(GtkButton* ok_button, gpointer user_data) {
    std::string host_ip = gtk_entry_get_text(host_ip_entry);
    std::string port = gtk_entry_get_text(port_entry);

    std::string addr = host_ip + ":" + port;

    download_certs(addr.c_str(), output_dir);
    quit();
}

static void on_cancel_button_clicked(GtkButton* sender, gpointer user_data) {
    quit();
}

static void quit() {
    gtk_widget_destroy(GTK_WIDGET(main_dialog));
}
