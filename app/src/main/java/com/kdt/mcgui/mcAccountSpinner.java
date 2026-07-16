package com.kdt.mcgui;

import static com.firefly.utils.ToastUtils.Toast;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.drawable.BitmapDrawable;
import android.graphics.drawable.Drawable;
import android.util.AttributeSet;
import android.util.Log;
import android.view.LayoutInflater;
import android.view.MotionEvent;
import android.view.View;
import android.view.ViewGroup;
import android.widget.AdapterView;
import android.widget.ArrayAdapter;
import android.widget.ImageView;
import android.widget.Toast;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.appcompat.app.AlertDialog;
import androidx.appcompat.widget.AppCompatSpinner;
import androidx.core.content.res.ResourcesCompat;

import com.externallogin.login.AuthResult;
import com.externallogin.login.OtherLoginApi;

import net.kdt.pojavlaunch.firefly.PojavApplication;
import net.kdt.pojavlaunch.firefly.PojavProfile;
import net.kdt.pojavlaunch.firefly.R;
import net.kdt.pojavlaunch.firefly.Tools;
import net.kdt.pojavlaunch.firefly.authenticator.microsoft.MicrosoftBackgroundLogin;
import net.kdt.pojavlaunch.firefly.authenticator.listener.DoneListener;
import net.kdt.pojavlaunch.firefly.authenticator.listener.ErrorListener;
import net.kdt.pojavlaunch.firefly.extra.ExtraConstants;
import net.kdt.pojavlaunch.firefly.extra.ExtraCore;
import net.kdt.pojavlaunch.firefly.extra.ExtraListener;
import net.kdt.pojavlaunch.firefly.value.MinecraftAccount;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Objects;

import fr.spse.extended_view.ExtendedTextView;

public class mcAccountSpinner extends AppCompatSpinner implements AdapterView.OnItemSelectedListener {
    public mcAccountSpinner(@NonNull Context context) {
        this(context, null);
    }

    public mcAccountSpinner(@NonNull Context context, @Nullable AttributeSet attrs) {
        super(context, attrs);
        init();
    }

    private final List<String> mAccountList = new ArrayList<>(2);
    private MinecraftAccount mSelectecAccount = null;

    /* Display the head of the current profile, here just to allow bitmap recycling */
    private BitmapDrawable mHeadDrawable;

    private final DoneListener mDoneListener = account -> {
        Toast(getContext(), R.string.main_login_done);

        // Check if the account being added is not one that is already existing
        // Like login twice on the same mc account...
        for (String mcAccountName : mAccountList) {
            if (mcAccountName.equals(account.username)) return;
        }

        mSelectecAccount = account;
        invalidate();
        mAccountList.add(account.username);
        reloadAccounts(false, mAccountList.size() - 1);
    };

    private final ErrorListener mErrorListener = errorMessage -> {
        Context context = getContext();
        Tools.showError(context, errorMessage);
        invalidate();
    };

    /* Triggered when we need to perform local login */
    private final ExtraListener<String[]> mLocalLoginListener = (key, value) -> {
        if (value[1].isEmpty()) { // Test mode
            MinecraftAccount account = new MinecraftAccount();
            account.username = value[0];
            try {
                account.save();
            } catch (IOException e) {
                Log.e("McAccountSpinner", "Failed to save the account : " + e);
            }

            mDoneListener.onLoginDone(account);
        }
        return false;
    };

    private final ExtraListener<MinecraftAccount> mOtherLoginListener = (key, value) -> {
        try {
            value.save();
        } catch (IOException e) {
            Log.e("McAccountSpinner", "Failed to save the account : " + e);
        }
        mDoneListener.onLoginDone(value);
        return false;
    };

    @SuppressLint("ClickableViewAccessibility")
    private void init() {
        // Set behavior
        reloadAccounts(true, 0);
        setOnItemSelectedListener(this);

        ExtraCore.addExtraListener(ExtraConstants.LOCAL_LOGIN_TODO, mLocalLoginListener);
        ExtraCore.addExtraListener(ExtraConstants.OTHER_LOGIN_TODO, mOtherLoginListener);
        ExtraCore.addExtraListener(ExtraConstants.MICROSOFT_LOGIN_TODO, mMicrosoftLoginListener);
    }

    /* Triggered when Microsoft login completes (account is already saved by LauncherActivity) */
    private final ExtraListener mMicrosoftLoginListener = (key, value) -> {
        if (!(value instanceof MinecraftAccount)) return false;
        mDoneListener.onLoginDone((MinecraftAccount) value);
        return false;
    };


    @Override
    public final void onItemSelected(AdapterView<?> parent, View view, int position, long id) {
        if (position == 0) {  // Add account button
            if (mAccountList.size() > 1) {
                ExtraCore.setValue(ExtraConstants.SELECT_AUTH_METHOD, true);
            }
            return;
        }

        pickAccount(position);
        if (mSelectecAccount != null) performLogin(mSelectecAccount);
    }

    @Override
    public final void onNothingSelected(AdapterView<?> parent) {
    }

    public void removeCurrentAccount(){
        removeAccount(getSelectedItemPosition());
    }

    private void removeAccount(int position) {
        if (position == 0) return;
        File accountFile = new File(Tools.DIR_ACCOUNT_NEW, mAccountList.get(position)+".json");
        if (accountFile.exists()) accountFile.delete();
        mAccountList.remove(position);

        reloadAccounts(false, 0);
    }

    /**
     * Allows checking whether we have an online account
     */
    public boolean isAccountOnline() {
        return mSelectecAccount != null && !mSelectecAccount.accessToken.equals("0");
    }

    public MinecraftAccount getSelectedAccount() {
        return mSelectecAccount;
    }

    @SuppressLint("ClickableViewAccessibility")
    private void setNoAccountBehavior() {
        // Set custom behavior when no account are present, to make it act as a button
        if (mAccountList.size() != 1) {
            // Remove any touch listener
            setOnTouchListener(null);
            return;
        }

        // Make the spinner act like a button, since there is no item to really select
        setOnTouchListener((v, event) -> {
            if (event.getAction() != MotionEvent.ACTION_UP) return false;
            // The activity should intercept this and spawn another fragment
            ExtraCore.setValue(ExtraConstants.SELECT_AUTH_METHOD, true);
            return true;
        });
    }

    /**
     * Reload the spinner, from memory or from scratch. A default account can be selected
     *
     * @param fromFiles        Whether we use files as the source of truth
     * @param overridePosition Force the spinner to be at this position, if not 0
     */
    private void reloadAccounts(boolean fromFiles, int overridePosition) {
        if (fromFiles) {
            mAccountList.clear();

            mAccountList.add(getContext().getString(R.string.main_add_account));
            File accountFolder = new File(Tools.DIR_ACCOUNT_NEW);
            if (accountFolder.exists()) {
                for (String fileName : accountFolder.list()) {
                    mAccountList.add(fileName.substring(0, fileName.length() - 5));
                }
            }
        }

        String[] accountArray = mAccountList.toArray(new String[0]);
        AccountAdapter accountAdapter = new AccountAdapter(getContext(), R.layout.item_minecraft_account, accountArray);
        accountAdapter.setDropDownViewResource(R.layout.item_minecraft_account);
        setAdapter(accountAdapter);

        // Pick what's available, might just be the the add account "button"
        pickAccount(overridePosition == 0 ? -1 : overridePosition);
        // if (mSelectecAccount != null) performLogin(mSelectecAccount);

        // Remove or add the behavior if needed
        setNoAccountBehavior();

    }

    private void performLogin(MinecraftAccount minecraftAccount) {
        if (minecraftAccount.isLocal()) return;
        if (minecraftAccount.isMicrosoft) {
            // Refresh Microsoft account token if expired
            if (System.currentTimeMillis() > minecraftAccount.expiresAt) {
                MicrosoftBackgroundLogin msLogin = new MicrosoftBackgroundLogin(true, minecraftAccount.msaRefreshToken);
                msLogin.performLogin(
                        null,
                        (DoneListener) account -> {
                            ExtraCore.setValue(ExtraConstants.MICROSOFT_LOGIN_TODO, account);
                        },
                        (ErrorListener) error -> {
                            mErrorListener.onLoginError(error);
                        }
                );
            }
            return;
        }
        if (!Objects.isNull(minecraftAccount.baseUrl) && !minecraftAccount.baseUrl.equals("0")) {
            OtherLoginApi.getINSTANCE().setBaseUrl(minecraftAccount.baseUrl);
            PojavApplication.sExecutorService.execute(() -> {
                try {
                    OtherLoginApi.getINSTANCE().refresh(minecraftAccount, false, new OtherLoginApi.Listener() {
                        @Override
                        public void onSuccess(AuthResult authResult) {
                            minecraftAccount.accessToken = authResult.getAccessToken();
                            ((Activity) getContext()).runOnUiThread(() -> {
                                ExtraCore.setValue(ExtraConstants.OTHER_LOGIN_TODO, minecraftAccount);
                            });
                        }

                        @Override
                        public void onFailed(String error) {
                            mErrorListener.onLoginError(new Throwable(error));
                        }
                    });
                } catch (IOException e) {
                    mErrorListener.onLoginError(e);
                }
            });
            return;
        }
    }

    /**
     * Pick the selected account, the one in settings if 0 is passed
     */
    private void pickAccount(int position) {
        MinecraftAccount selectedAccount;
        if (position != -1) {
            PojavProfile.setCurrentProfile(getContext(), mAccountList.get(position));
            selectedAccount = PojavProfile.getCurrentProfileContent(getContext(), mAccountList.get(position));


            // WORKAROUND
            // Account file corrupted due to previous versions having improper encoding
            if (selectedAccount == null) {
                removeCurrentAccount();
                pickAccount(-1);
                setSelection(0);
                return;
            }
            setSelection(position);
        } else {
            // Get the current profile, or the first available profile if the wanted one is unavailable
            selectedAccount = PojavProfile.getCurrentProfileContent(getContext(), null);
            int spinnerPosition = selectedAccount == null
                    ? mAccountList.size() <= 1 ? 0 : 1
                    : mAccountList.indexOf(selectedAccount.username);
            setSelection(spinnerPosition, false);
        }

        mSelectecAccount = selectedAccount;
        setImageFromSelectedAccount();
    }

    @Deprecated()
    /* Legacy behavior, update the head image manually for the selected account */
    private void setImageFromSelectedAccount() {
        BitmapDrawable oldBitmapDrawable = mHeadDrawable;

        if (mSelectecAccount != null) {
            View layout = getSelectedView();
            if (layout != null){
                ExtendedTextView view = layout.findViewById(R.id.account_item);
                Bitmap bitmap = mSelectecAccount.getSkinFace();
                if (bitmap != null) {
                    mHeadDrawable = new BitmapDrawable(getResources(), bitmap);

                    view.setCompoundDrawables(mHeadDrawable, null, null, null);
                } else {
                    view.setCompoundDrawables(null, null, null, null);
                }
                view.postProcessDrawables();
            }
        }

        if (oldBitmapDrawable != null) {
            oldBitmapDrawable.getBitmap().recycle();
        }
    }


    private class AccountAdapter extends ArrayAdapter<String> {

        private final HashMap<String, Drawable> mImageCache = new HashMap<>();

        public AccountAdapter(@NonNull Context context, int resource, @NonNull String[] objects) {
            super(context, resource, objects);
        }

        @Override
        public View getDropDownView(int position, @Nullable View convertView, @NonNull ViewGroup parent) {
            if (convertView == null) {
                convertView = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_minecraft_account, parent, false);
            }
            ExtendedTextView textview = convertView.findViewById(R.id.account_item);
            ImageView deleteButton = convertView.findViewById(R.id.delete_account_button);
            textview.setText(super.getItem(position));

            // Handle the "Add account section"
            if (position == 0) {
                textview.setCompoundDrawables(ResourcesCompat.getDrawable(parent.getResources(), R.drawable.ic_add, null), null, null, null);
                deleteButton.setVisibility(View.GONE);
            }
            else {
                String username = super.getItem(position);
                Drawable accountHead = mImageCache.get(username);
                if (accountHead == null) {
                    accountHead = new BitmapDrawable(parent.getResources(), MinecraftAccount.getSkinFace(username));
                    mImageCache.put(username, accountHead);
                }
                textview.setCompoundDrawables(accountHead, null, null, null);
                deleteButton.setVisibility(View.VISIBLE);
                deleteButton.setOnClickListener(v -> {
                    showDeleteDialog(getContext(), position);
                });
            }
            return convertView;
        }

        @NonNull
        @Override
        public View getView(int position, View convertView, @NonNull ViewGroup parent) {
            View view = getDropDownView(position, convertView, parent);
            view.findViewById(R.id.delete_account_button).setVisibility(View.GONE);
            return view;
        }

        private void showDeleteDialog(Context context, int position) {
            new AlertDialog.Builder(context)
                    .setMessage(R.string.warning_remove_account)
                    .setPositiveButton(android.R.string.cancel, null)
                    .setNeutralButton(R.string.global_delete, (dialog, which) -> {
                        onDetachedFromWindow();
                        removeAccount(position);
                    })
                    .show();
        }
    }
}
