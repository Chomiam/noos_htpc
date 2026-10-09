#define _GNU_SOURCE
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <pthread.h>
#include <errno.h>
#include <dirent.h>
#include <sys/ioctl.h>
#include <sys/socket.h>
#include <sys/un.h>
#include <sys/epoll.h>
#include <linux/input.h>
#include <linux/uinput.h>

#include <gtk/gtk.h>
#include <gtk-layer-shell/gtk-layer-shell.h>
#include <gio/gio.h>

/* ========================================================================= */
/* CONFIGURATION & TYPES                                                     */
/* ========================================================================= */

typedef enum {
    LAYOUT_AZERTY,
    LAYOUT_QWERTY
} KeyboardLayout;

typedef enum {
    MODE_LOWER,
    MODE_UPPER,
    MODE_SYMBOLS
} KeyboardMode;

typedef struct {
    const char *label_lower;
    const char *label_upper;
    const char *label_sym;
    int keycode;
    int shift_sym; // 1 si le symbole nécessite shift
    int width_weight; // largeur relative
} KeyDef;

#define MAX_ROWS 4
#define MAX_COLS 14

static GtkWidget *window = NULL;
static GtkWidget *key_buttons[MAX_ROWS][MAX_COLS];
static int row_sizes[MAX_ROWS];
static int cur_row = 0;
static int cur_col = 0;
static gboolean is_visible = FALSE;

static KeyboardLayout current_layout = LAYOUT_AZERTY;
static KeyboardMode current_mode = MODE_LOWER;

static int uinput_fd = -1;
static char sock_path[256];

/* ========================================================================= */
/* DEFINITIONS DES TOUCHES                                                   */
/* ========================================================================= */

/* Touches spéciales */
#define KEY_CODE_SHIFT    -1
#define KEY_CODE_SYMBOLS  -2
#define KEY_CODE_COLLAPSE -3

/* Disposition AZERTY */
static const KeyDef azerty_rows[MAX_ROWS][MAX_COLS] = {
    /* Ligne 0 : 10 lettres */
    {
        {"a", "A", "1", KEY_A, 0, 1}, {"z", "Z", "2", KEY_Z, 0, 1},
        {"e", "E", "3", KEY_E, 0, 1}, {"r", "R", "4", KEY_R, 0, 1},
        {"t", "T", "5", KEY_T, 0, 1}, {"y", "Y", "6", KEY_Y, 0, 1},
        {"u", "U", "7", KEY_U, 0, 1}, {"i", "I", "8", KEY_I, 0, 1},
        {"o", "O", "9", KEY_O, 0, 1}, {"p", "P", "0", KEY_P, 0, 1}
    },
    /* Ligne 1 : 10 lettres */
    {
        {"q", "Q", "@", KEY_Q, 0, 1}, {"s", "S", "#", KEY_S, 0, 1},
        {"d", "D", "$", KEY_D, 0, 1}, {"f", "F", "%", KEY_F, 0, 1},
        {"g", "G", "&", KEY_G, 0, 1}, {"h", "H", "*", KEY_H, 0, 1},
        {"j", "J", "-", KEY_J, 0, 1}, {"k", "K", "+", KEY_K, 0, 1},
        {"l", "L", "(", KEY_L, 0, 1}, {"m", "M", ")", KEY_M, 0, 1}
    },
    /* Ligne 2 : 10 lettres / ponctuations */
    {
        {"w", "W", "!", KEY_W, 0, 1}, {"x", "X", "?", KEY_X, 0, 1},
        {"c", "C", "_", KEY_C, 0, 1}, {"v", "V", "\"", KEY_V, 0, 1},
        {"b", "B", "'", KEY_B, 0, 1}, {"n", "N", ";", KEY_N, 0, 1},
        {":", ":", ":", KEY_DOT, 1, 1}, {"/", "/", "/", KEY_SLASH, 0, 1},
        {".", ".", ".", KEY_DOT, 0, 1}, {"-", "-", "=", KEY_MINUS, 0, 1}
    },
    /* Ligne 3 : Contrôles */
    {
        {"Maj", "Maj", "Maj", KEY_CODE_SHIFT, 0, 2},
        {"?123", "?123", "ABC", KEY_CODE_SYMBOLS, 0, 2},
        {"Espace", "Espace", "Espace", KEY_SPACE, 0, 4},
        {"Effacer", "Effacer", "Effacer", KEY_BACKSPACE, 0, 2},
        {"Valider", "Valider", "Valider", KEY_ENTER, 0, 2},
        {"▼ Réduire", "▼ Réduire", "▼ Réduire", KEY_CODE_COLLAPSE, 0, 2}
    }
};

/* Disposition QWERTY */
static const KeyDef qwerty_rows[MAX_ROWS][MAX_COLS] = {
    {
        {"q", "Q", "1", KEY_Q, 0, 1}, {"w", "W", "2", KEY_W, 0, 1},
        {"e", "E", "3", KEY_E, 0, 1}, {"r", "R", "4", KEY_R, 0, 1},
        {"t", "T", "5", KEY_T, 0, 1}, {"y", "Y", "6", KEY_Y, 0, 1},
        {"u", "U", "7", KEY_U, 0, 1}, {"i", "I", "8", KEY_I, 0, 1},
        {"o", "O", "9", KEY_O, 0, 1}, {"p", "P", "0", KEY_P, 0, 1}
    },
    {
        {"a", "A", "@", KEY_A, 0, 1}, {"s", "S", "#", KEY_S, 0, 1},
        {"d", "D", "$", KEY_D, 0, 1}, {"f", "F", "%", KEY_F, 0, 1},
        {"g", "G", "&", KEY_G, 0, 1}, {"h", "H", "*", KEY_H, 0, 1},
        {"j", "J", "-", KEY_J, 0, 1}, {"k", "K", "+", KEY_K, 0, 1},
        {"l", "L", "(", KEY_L, 0, 1}, {":", ":", ")", KEY_DOT, 1, 1}
    },
    {
        {"z", "Z", "!", KEY_Z, 0, 1}, {"x", "X", "?", KEY_X, 0, 1},
        {"c", "C", "_", KEY_C, 0, 1}, {"v", "V", "\"", KEY_V, 0, 1},
        {"b", "B", "'", KEY_B, 0, 1}, {"n", "N", ";", KEY_N, 0, 1},
        {"m", "M", "/", KEY_M, 0, 1}, {".", ".", ".", KEY_DOT, 0, 1},
        {"/", "/", "/", KEY_SLASH, 0, 1}, {"-", "-", "=", KEY_MINUS, 0, 1}
    },
    {
        {"Maj", "Maj", "Maj", KEY_CODE_SHIFT, 0, 2},
        {"?123", "?123", "ABC", KEY_CODE_SYMBOLS, 0, 2},
        {"Espace", "Espace", "Espace", KEY_SPACE, 0, 4},
        {"Effacer", "Effacer", "Effacer", KEY_BACKSPACE, 0, 2},
        {"Valider", "Valider", "Valider", KEY_ENTER, 0, 2},
        {"▼ Réduire", "▼ Réduire", "▼ Réduire", KEY_CODE_COLLAPSE, 0, 2}
    }
};

/* ========================================================================= */
/* UINPUT : EMULATION CLAVIER LINUX KERNEL                                   */
/* ========================================================================= */

static int init_uinput(void) {
    int fd = open("/dev/uinput", O_WRONLY | O_NONBLOCK);
    if (fd < 0) {
        perror("[Noos OSK] Impossible d'ouvrir /dev/uinput");
        return -1;
    }

    ioctl(fd, UI_SET_EVBIT, EV_KEY);
    ioctl(fd, UI_SET_EVBIT, EV_SYN);

    for (int i = 0; i < 255; i++) {
        ioctl(fd, UI_SET_KEYBIT, i);
    }

    struct uinput_setup usetup;
    memset(&usetup, 0, sizeof(usetup));
    usetup.id.bustype = BUS_USB;
    usetup.id.vendor = 0x1337;
    usetup.id.product = 0x0001;
    strcpy(usetup.name, "Noos TV Gamepad OSK");

    if (ioctl(fd, UI_DEV_SETUP, &usetup) < 0) {
        perror("[Noos OSK] Erreur UI_DEV_SETUP");
        close(fd);
        return -1;
    }

    if (ioctl(fd, UI_DEV_CREATE) < 0) {
        perror("[Noos OSK] Erreur UI_DEV_CREATE");
        close(fd);
        return -1;
    }

    printf("[Noos OSK] Périphérique virtuel /dev/uinput initialisé avec succès.\n");
    return fd;
}

static void emit_uinput_key(int fd, int code, int shift) {
    if (fd < 0 || code <= 0) return;

    struct input_event ev[6];
    int count = 0;

    if (shift) {
        ev[count].type = EV_KEY; ev[count].code = KEY_LEFTSHIFT; ev[count].value = 1; count++;
        ev[count].type = EV_SYN; ev[count].code = SYN_REPORT; ev[count].value = 0; count++;
    }

    ev[count].type = EV_KEY; ev[count].code = code; ev[count].value = 1; count++;
    ev[count].type = EV_SYN; ev[count].code = SYN_REPORT; ev[count].value = 0; count++;

    ev[count].type = EV_KEY; ev[count].code = code; ev[count].value = 0; count++;
    ev[count].type = EV_SYN; ev[count].code = SYN_REPORT; ev[count].value = 0; count++;

    if (shift) {
        ev[count].type = EV_KEY; ev[count].code = KEY_LEFTSHIFT; ev[count].value = 0; count++;
        ev[count].type = EV_SYN; ev[count].code = SYN_REPORT; ev[count].value = 0; count++;
    }

    write(fd, ev, sizeof(struct input_event) * count);
}

/* ========================================================================= */
/* GESTION DE L'INTERFACE GRAPHIQUE GTK                                      */
/* ========================================================================= */

static void update_key_labels(void);

static void show_keyboard(void) {
    if (!window || is_visible) return;
    is_visible = TRUE;
    gtk_widget_show_all(window);
    printf("[Noos OSK] Clavier virtuel affiché en overlay.\n");
}

static void hide_keyboard(void) {
    if (!window || !is_visible) return;
    is_visible = FALSE;
    gtk_widget_hide(window);
    printf("[Noos OSK] Clavier virtuel réduit / masqué.\n");
}

static void toggle_keyboard(void) {
    if (is_visible) hide_keyboard();
    else show_keyboard();
}

static gboolean update_focus_ui(gpointer user_data) {
    (void)user_data;
    for (int r = 0; r < MAX_ROWS; r++) {
        for (int c = 0; c < row_sizes[r]; c++) {
            if (key_buttons[r][c]) {
                GtkStyleContext *ctx = gtk_widget_get_style_context(key_buttons[r][c]);
                if (r == cur_row && c == cur_col) {
                    gtk_style_context_add_class(ctx, "key-focused");
                } else {
                    gtk_style_context_remove_class(ctx, "key-focused");
                }
            }
        }
    }
    return FALSE;
}

static void press_current_key(void) {
    const KeyDef (*rows)[MAX_COLS] = (current_layout == LAYOUT_AZERTY) ? azerty_rows : qwerty_rows;
    if (cur_row < 0 || cur_row >= MAX_ROWS || cur_col < 0 || cur_col >= row_sizes[cur_row]) return;

    KeyDef k = rows[cur_row][cur_col];

    if (k.keycode == KEY_CODE_COLLAPSE) {
        hide_keyboard();
        return;
    }

    if (k.keycode == KEY_CODE_SHIFT) {
        current_mode = (current_mode == MODE_UPPER) ? MODE_LOWER : MODE_UPPER;
        update_key_labels();
        return;
    }

    if (k.keycode == KEY_CODE_SYMBOLS) {
        current_mode = (current_mode == MODE_SYMBOLS) ? MODE_LOWER : MODE_SYMBOLS;
        update_key_labels();
        return;
    }

    int shift = (current_mode == MODE_UPPER) ? 1 : 0;
    emit_uinput_key(uinput_fd, k.keycode, shift);
}

static void on_key_clicked(GtkWidget *widget, gpointer data) {
    (void)widget;
    int idx = GPOINTER_TO_INT(data);
    cur_row = idx / 100;
    cur_col = idx % 100;
    g_idle_add(update_focus_ui, NULL);
    press_current_key();
}

static void update_key_labels(void) {
    const KeyDef (*rows)[MAX_COLS] = (current_layout == LAYOUT_AZERTY) ? azerty_rows : qwerty_rows;
    for (int r = 0; r < MAX_ROWS; r++) {
        for (int c = 0; c < row_sizes[r]; c++) {
            if (!key_buttons[r][c]) continue;
            KeyDef k = rows[r][c];
            const char *txt = k.label_lower;
            if (current_mode == MODE_UPPER) txt = k.label_upper;
            else if (current_mode == MODE_SYMBOLS) txt = k.label_sym;
            gtk_button_set_label(GTK_BUTTON(key_buttons[r][c]), txt);
        }
    }
}

static void load_config_layout(void) {
    const char *home = getenv("HOME");
    if (!home) home = "/home/noos";

    char conf_path[512];
    snprintf(conf_path, sizeof(conf_path), "%s/.config/noos-htpc/keyboard.json", home);

    FILE *f = fopen(conf_path, "r");
    if (f) {
        char buf[256];
        if (fgets(buf, sizeof(buf), f)) {
            if (strstr(buf, "qwerty") || strstr(buf, "QWERTY")) {
                current_layout = LAYOUT_QWERTY;
            } else {
                current_layout = LAYOUT_AZERTY;
            }
        }
        fclose(f);
    }
}

static void create_keyboard_ui(void) {
    row_sizes[0] = 10;
    row_sizes[1] = 10;
    row_sizes[2] = 10;
    row_sizes[3] = 6;

    window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(window), "Noos TV Gamepad OSK");
    gtk_window_set_decorated(GTK_WINDOW(window), FALSE);

    /* Configuration Layer Shell pour Wayland (KWin / Cage) */
    gtk_layer_init_for_window(GTK_WINDOW(window));
    gtk_layer_set_layer(GTK_WINDOW(window), GTK_LAYER_SHELL_LAYER_OVERLAY);
    gtk_layer_set_anchor(GTK_WINDOW(window), GTK_LAYER_SHELL_EDGE_BOTTOM, TRUE);
    gtk_layer_set_anchor(GTK_WINDOW(window), GTK_LAYER_SHELL_EDGE_LEFT, TRUE);
    gtk_layer_set_anchor(GTK_WINDOW(window), GTK_LAYER_SHELL_EDGE_RIGHT, TRUE);
    gtk_layer_set_keyboard_mode(GTK_WINDOW(window), GTK_LAYER_SHELL_KEYBOARD_MODE_NONE);
    gtk_layer_set_exclusive_zone(GTK_WINDOW(window), 0);

    GtkWidget *main_box = gtk_box_new(GTK_ORIENTATION_VERTICAL, 6);
    gtk_style_context_add_class(gtk_widget_get_style_context(main_box), "osk-container");
    gtk_container_add(GTK_CONTAINER(window), main_box);

    /* Barre supérieure avec indicateur et disposition */
    GtkWidget *top_bar = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 12);
    gtk_style_context_add_class(gtk_widget_get_style_context(top_bar), "osk-topbar");

    GtkWidget *brand_lbl = gtk_label_new("NOOS TV • CLAVIER VIRTUEL");
    gtk_style_context_add_class(gtk_widget_get_style_context(brand_lbl), "osk-brand");
    gtk_box_pack_start(GTK_BOX(top_bar), brand_lbl, FALSE, FALSE, 0);

    GtkWidget *hint_lbl = gtk_label_new("(A) Taper   (X) Effacer   (Y) Espace   (B / ↓) Réduire   (Start) Valider");
    gtk_style_context_add_class(gtk_widget_get_style_context(hint_lbl), "osk-hints");
    gtk_box_pack_end(GTK_BOX(top_bar), hint_lbl, FALSE, FALSE, 0);

    gtk_box_pack_start(GTK_BOX(main_box), top_bar, FALSE, FALSE, 2);

    /* Grille des touches */
    const KeyDef (*rows)[MAX_COLS] = (current_layout == LAYOUT_AZERTY) ? azerty_rows : qwerty_rows;

    for (int r = 0; r < MAX_ROWS; r++) {
        GtkWidget *row_box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
        gtk_widget_set_halign(row_box, GTK_ALIGN_CENTER);

        for (int c = 0; c < row_sizes[r]; c++) {
            KeyDef k = rows[r][c];
            GtkWidget *btn = gtk_button_new_with_label(k.label_lower);
            gtk_style_context_add_class(gtk_widget_get_style_context(btn), "osk-key");
            
            if (k.width_weight > 1) {
                gtk_style_context_add_class(gtk_widget_get_style_context(btn), "key-wide");
            }
            if (k.keycode == KEY_CODE_COLLAPSE) {
                gtk_style_context_add_class(gtk_widget_get_style_context(btn), "key-collapse");
            } else if (k.keycode == KEY_ENTER) {
                gtk_style_context_add_class(gtk_widget_get_style_context(btn), "key-enter");
            }

            int id = r * 100 + c;
            g_signal_connect(btn, "clicked", G_CALLBACK(on_key_clicked), GINT_TO_POINTER(id));

            key_buttons[r][c] = btn;
            gtk_box_pack_start(GTK_BOX(row_box), btn, FALSE, FALSE, 0);
        }
        gtk_box_pack_start(GTK_BOX(main_box), row_box, FALSE, FALSE, 0);
    }

    /* CSS Styling élégant Catppuccin / Glassmorphism */
    GtkCssProvider *provider = gtk_css_provider_new();
    const char *css =
        ".osk-container { background: rgba(8, 12, 20, 0.94); padding: 12px 24px 18px 24px; border-top: 2px solid #38bdf8; box-shadow: 0 -10px 40px rgba(0,0,0,0.8); }"
        ".osk-topbar { padding-bottom: 8px; border-bottom: 1px solid rgba(255,255,255,0.08); margin-bottom: 8px; }"
        ".osk-brand { font-size: 13px; font-weight: 700; color: #38bdf8; letter-spacing: 1px; }"
        ".osk-hints { font-size: 12px; color: #94a3b8; font-weight: 500; }"
        ".osk-key { min-width: 54px; min-height: 48px; border-radius: 8px; background: rgba(30, 41, 59, 0.85); color: #f8fafc; font-size: 18px; font-weight: 600; border: 1px solid rgba(255,255,255,0.08); box-shadow: 0 4px 6px rgba(0,0,0,0.3); transition: all 150ms ease; }"
        ".osk-key:hover { background: rgba(51, 65, 85, 0.95); border-color: rgba(255,255,255,0.2); }"
        ".key-wide { min-width: 90px; font-size: 14px; }"
        ".key-collapse { background: rgba(239, 68, 68, 0.2); border-color: rgba(239, 68, 68, 0.4); color: #fca5a5; }"
        ".key-enter { background: rgba(16, 185, 129, 0.25); border-color: rgba(16, 185, 129, 0.5); color: #6ee7b7; }"
        ".key-focused { background: #38bdf8 !important; color: #020617 !important; border-color: #ffffff !important; box-shadow: 0 0 18px #38bdf8, 0 0 30px rgba(56, 189, 248, 0.6) !important; transform: scale(1.08); }";

    gtk_css_provider_load_from_data(provider, css, -1, NULL);
    gtk_style_context_add_provider_for_screen(gdk_screen_get_default(), GTK_STYLE_PROVIDER(provider), GTK_STYLE_PROVIDER_PRIORITY_APPLICATION);

    g_idle_add(update_focus_ui, NULL);
}

/* ========================================================================= */
/* NAVIGATION MANETTE (EVDEV /DEV/INPUT/EVENT*)                              */
/* ========================================================================= */

static void *gamepad_thread_fn(void *arg) {
    (void)arg;
    int epoll_fd = epoll_create1(0);

    DIR *d = opendir("/dev/input");
    if (!d) return NULL;

    struct dirent *dir;
    while ((dir = readdir(d)) != NULL) {
        if (strncmp(dir->d_name, "event", 5) == 0) {
            char path[128];
            snprintf(path, sizeof(path), "/dev/input/%s", dir->d_name);
            int fd = open(path, O_RDONLY | O_NONBLOCK);
            if (fd >= 0) {
                unsigned long evbits = 0;
                ioctl(fd, EVIOCGBIT(0, sizeof(evbits)), &evbits);
                if (evbits & (1 << EV_KEY)) {
                    struct epoll_event ev;
                    ev.events = EPOLLIN;
                    ev.data.fd = fd;
                    epoll_ctl(epoll_fd, EPOLL_CTL_ADD, fd, &ev);
                } else {
                    close(fd);
                }
            }
        }
    }
    closedir(d);

    struct epoll_event events[16];
    struct input_event ie;

    while (1) {
        int n = epoll_wait(epoll_fd, events, 16, 100);
        for (int i = 0; i < n; i++) {
            int fd = events[i].data.fd;
            while (read(fd, &ie, sizeof(ie)) == sizeof(ie)) {
                if (ie.type == EV_KEY && ie.value == 1) {
                    /* Si le clavier n'est pas affiché, vérifier raccourci d'activation */
                    if (!is_visible) {
                        /* Bouton Select / Share (314) pour forcer l'affichage */
                        if (ie.code == 314) {
                            g_idle_add((GSourceFunc)show_keyboard, NULL);
                        }
                        continue;
                    }

                    /* Clavier affiché : contrôle complet */
                    if (ie.code == KEY_LEFT || ie.code == BTN_DPAD_LEFT) {
                        if (cur_col > 0) cur_col--;
                        else cur_col = row_sizes[cur_row] - 1;
                        g_idle_add(update_focus_ui, NULL);
                    } else if (ie.code == KEY_RIGHT || ie.code == BTN_DPAD_RIGHT) {
                        if (cur_col < row_sizes[cur_row] - 1) cur_col++;
                        else cur_col = 0;
                        g_idle_add(update_focus_ui, NULL);
                    } else if (ie.code == KEY_UP || ie.code == BTN_DPAD_UP) {
                        if (cur_row > 0) {
                            cur_row--;
                            if (cur_col >= row_sizes[cur_row]) cur_col = row_sizes[cur_row] - 1;
                            g_idle_add(update_focus_ui, NULL);
                        }
                    } else if (ie.code == KEY_DOWN || ie.code == BTN_DPAD_DOWN) {
                        if (cur_row < MAX_ROWS - 1) {
                            cur_row++;
                            if (cur_col >= row_sizes[cur_row]) cur_col = row_sizes[cur_row] - 1;
                            g_idle_add(update_focus_ui, NULL);
                        } else {
                            /* Flèche du bas sur la dernière ligne = réduire le clavier ! */
                            g_idle_add((GSourceFunc)hide_keyboard, NULL);
                        }
                    } else if (ie.code == BTN_SOUTH || ie.code == BTN_A || ie.code == KEY_ENTER) {
                        /* Taper la touche sélectionnée */
                        press_current_key();
                    } else if (ie.code == BTN_EAST || ie.code == BTN_B || ie.code == KEY_ESC) {
                        /* Réduire / Masquer le clavier */
                        g_idle_add((GSourceFunc)hide_keyboard, NULL);
                    } else if (ie.code == BTN_WEST || ie.code == BTN_X) {
                        /* Effacer (Backspace) */
                        emit_uinput_key(uinput_fd, KEY_BACKSPACE, 0);
                    } else if (ie.code == BTN_NORTH || ie.code == BTN_Y) {
                        /* Espace */
                        emit_uinput_key(uinput_fd, KEY_SPACE, 0);
                    } else if (ie.code == BTN_START) {
                        /* Valider (Entrée) */
                        emit_uinput_key(uinput_fd, KEY_ENTER, 0);
                    } else if (ie.code == BTN_TL || ie.code == BTN_TR) {
                        /* Bascule Maj / Chiffres */
                        current_mode = (current_mode == MODE_LOWER) ? MODE_UPPER : MODE_LOWER;
                        update_key_labels();
                    }
                }
            }
        }
    }
    return NULL;
}

/* ========================================================================= */
/* ECOUTE UNIVERSELLE AT-SPI2 & KWIN DBUS (DETECTION DES CHAMPS TEXTES)     */
/* ========================================================================= */

static void on_atspi_state_changed(GDBusConnection *conn,
                                   const gchar *sender,
                                   const gchar *object_path,
                                   const gchar *interface_name,
                                   const gchar *signal_name,
                                   GVariant *parameters,
                                   gpointer user_data) {
    (void)conn; (void)sender; (void)object_path; (void)interface_name; (void)signal_name; (void)user_data;
    
    const gchar *state = NULL;
    gint val1 = 0;
    gint val2 = 0;
    GVariant *any_val = NULL;

    g_variant_get(parameters, "(&sii@v)", &state, &val1, &val2, &any_val);

    if (g_strcmp0(state, "focused") == 0) {
        if (val1 == 1) {
            printf("[Noos OSK] Champ de texte détecté et activé (AT-SPI2) sur : %s\n", object_path);
            g_idle_add((GSourceFunc)show_keyboard, NULL);
        } else if (val1 == 0) {
            /* Ne pas masquer immédiatement pour permettre la navigation entre champs */
        }
    }

    if (any_val) g_variant_unref(any_val);
}

static void on_kwin_vk_signal(GDBusConnection *conn,
                              const gchar *sender,
                              const gchar *object_path,
                              const gchar *interface_name,
                              const gchar *signal_name,
                              GVariant *parameters,
                              gpointer user_data) {
    (void)conn; (void)sender; (void)object_path; (void)interface_name; (void)user_data;
    
    gboolean active = FALSE;
    if (g_strcmp0(signal_name, "visibleChanged") == 0 || g_strcmp0(signal_name, "activeChanged") == 0) {
        g_variant_get(parameters, "(b)", &active);
        printf("[Noos OSK] Signal KWin VirtualKeyboard : %s = %d\n", signal_name, active);
        if (active) g_idle_add((GSourceFunc)show_keyboard, NULL);
        else g_idle_add((GSourceFunc)hide_keyboard, NULL);
    }
}

static void *dbus_listener_thread_fn(void *arg) {
    (void)arg;
    GError *error = NULL;

    /* 1. Connexion au Bus Session pour KWin */
    GDBusConnection *session_conn = g_bus_get_sync(G_BUS_TYPE_SESSION, NULL, &error);
    if (session_conn) {
        g_dbus_connection_signal_subscribe(
            session_conn,
            "org.kde.KWin",
            "org.kde.kwin.VirtualKeyboard",
            NULL,
            "/VirtualKeyboard",
            NULL,
            G_DBUS_SIGNAL_FLAGS_NONE,
            on_kwin_vk_signal,
            NULL,
            NULL
        );
        printf("[Noos OSK] Écoute des signaux KWin VirtualKeyboard active.\n");
    }

    /* 2. Connexion au Bus Accessibilité AT-SPI2 */
    const char *atspi_addr = g_getenv("AT_SPI_BUS_ADDRESS");
    char fallback_path[128];
    if (!atspi_addr || strlen(atspi_addr) == 0) {
        snprintf(fallback_path, sizeof(fallback_path), "unix:path=/run/user/%d/at-spi/bus", getuid());
        atspi_addr = fallback_path;
    }

    GDBusConnection *a11y_conn = g_dbus_connection_new_for_address_sync(
        atspi_addr,
        G_DBUS_CONNECTION_FLAGS_AUTHENTICATION_CLIENT,
        NULL,
        NULL,
        NULL
    );

    if (a11y_conn) {
        g_dbus_connection_signal_subscribe(
            a11y_conn,
            NULL,
            "org.a11y.atspi.Event.Object",
            "StateChanged",
            NULL,
            "focused",
            G_DBUS_SIGNAL_FLAGS_NONE,
            on_atspi_state_changed,
            NULL,
            NULL
        );
        printf("[Noos OSK] Écoute AT-SPI2 active pour détection universelle des champs textes.\n");
    }

    return NULL;
}

/* ========================================================================= */
/* SOCKET UNIX POUR COMMANDES DASHBOARD & CLI                                */
/* ========================================================================= */

static void *socket_server_thread_fn(void *arg) {
    (void)arg;
    int sfd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sfd < 0) return NULL;

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, sock_path, sizeof(addr.sun_path) - 1);
    unlink(sock_path);

    if (bind(sfd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        close(sfd);
        return NULL;
    }

    listen(sfd, 5);

    while (1) {
        int cfd = accept(sfd, NULL, NULL);
        if (cfd >= 0) {
            char buf[128];
            int r = read(cfd, buf, sizeof(buf) - 1);
            if (r > 0) {
                buf[r] = '\0';
                if (strstr(buf, "SHOW")) g_idle_add((GSourceFunc)show_keyboard, NULL);
                else if (strstr(buf, "HIDE")) g_idle_add((GSourceFunc)hide_keyboard, NULL);
                else if (strstr(buf, "TOGGLE")) g_idle_add((GSourceFunc)toggle_keyboard, NULL);
                else if (strstr(buf, "SET_LAYOUT qwerty")) {
                    current_layout = LAYOUT_QWERTY;
                    g_idle_add((GSourceFunc)update_key_labels, NULL);
                } else if (strstr(buf, "SET_LAYOUT azerty")) {
                    current_layout = LAYOUT_AZERTY;
                    g_idle_add((GSourceFunc)update_key_labels, NULL);
                }
            }
            close(cfd);
        }
    }
    return NULL;
}

/* ========================================================================= */
/* MAIN ENTRY POINT                                                          */
/* ========================================================================= */

int main(int argc, char *argv[]) {
    snprintf(sock_path, sizeof(sock_path), "/run/user/%d/noos-osk.sock", getuid());

    /* Si appelé avec des arguments CLI (--show, --hide, --toggle) */
    if (argc > 1) {
        int s = socket(AF_UNIX, SOCK_STREAM, 0);
        if (s >= 0) {
            struct sockaddr_un addr;
            memset(&addr, 0, sizeof(addr));
            addr.sun_family = AF_UNIX;
            strncpy(addr.sun_path, sock_path, sizeof(addr.sun_path) - 1);
            if (connect(s, (struct sockaddr *)&addr, sizeof(addr)) == 0) {
                if (strcmp(argv[1], "--show") == 0) write(s, "SHOW", 4);
                else if (strcmp(argv[1], "--hide") == 0) write(s, "HIDE", 4);
                else if (strcmp(argv[1], "--toggle") == 0) write(s, "TOGGLE", 6);
                close(s);
                return 0;
            }
            close(s);
        }
    }

    gtk_init(&argc, &argv);

    load_config_layout();
    uinput_fd = init_uinput();

    create_keyboard_ui();

    /* Démarrage des threads d'écoute */
    pthread_t th_gamepad, th_dbus, th_sock;
    pthread_create(&th_gamepad, NULL, gamepad_thread_fn, NULL);
    pthread_create(&th_dbus, NULL, dbus_listener_thread_fn, NULL);
    pthread_create(&th_sock, NULL, socket_server_thread_fn, NULL);

    /* Initialement masqué */
    hide_keyboard();

    gtk_main();

    if (uinput_fd >= 0) {
        ioctl(uinput_fd, UI_DEV_DESTROY);
        close(uinput_fd);
    }
    unlink(sock_path);
    return 0;
}
